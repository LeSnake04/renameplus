use std::path::PathBuf;

use crate::{args::OnConflict, config::Config, rename::Rename};
use anyhow::{anyhow, ensure, Context, Result};
use log::debug;

impl Rename {
	pub fn try_new(m: clap::ArgMatches, config: Config) -> Result<Self> {
		debug!("Parsing input args");
		let files: Vec<PathBuf> = m
			.try_get_many::<PathBuf>("file")?
			.context("Failed to get files")
			.map(move |v| v.cloned().collect())?;
		let output_files: Option<Vec<PathBuf>> = m
			.try_get_many::<PathBuf>("output-files")
			.context("failed to get argument \'output-files\'")?
			.map(move |v| v.cloned().collect());
		let replace_sets: Vec<String> = {
			let sets_opt: Option<Vec<_>> = m
				.try_get_many::<String>("sets")
				.context("failed to get argument \'sets\'")?
				.map(move |v| v.cloned().collect());
			let mut out = vec![];
			if let Some(sets) = sets_opt {
				for set in sets {
					match config.sets.get(&set).is_some() {
						true => out.push(set),
						false => Err(anyhow!("Set \"{set}\" not found"))?,
					}
				}
			}
			out
		};
		if let Some(ref o) = output_files {
			ensure!(o.is_empty(), "Need at least one output file");
			let output_file_last = o.last().context("output_files is empty")?;
			if o.len() < files.len() && !output_file_last.exists() {
				Err(anyhow!("Last entry of array does not exist"))?;
				ensure!(output_file_last.is_dir(), "Last entry isn't a directory");
			}
		}
		let undo_on_err: bool = *m
			.try_get_one("undo-on-err")
			.context("Failed to get argument \'undo-on-errr\'")?
			.ok_or(anyhow!("undo_on_err not set"))?;
		let replace: Vec<(String, String)> = {
			let mut out: Vec<(String, String)> = vec![];
			let inputs: Vec<String> = match m
				.try_get_many::<String>("replace")
				.context("Failed to get arg replace")?
			{
				Some(r) => r.cloned().collect(),
				None => vec![],
			};
			for inp in inputs {
				let splits: Vec<&str> = inp.split('/').collect();
				out.push(match splits.len() {
					1 => (splits[0].to_string(), "".to_string()),
					2 => (splits[0].to_string(), splits[1].to_string()),
					_ => Err(anyhow!("{inp} can only have one or two '/'"))?,
				})
			}
			out
		};
		Ok(Self {
			undo_on_err,
			fragile: m
				.try_contains_id("fragile")
				.context("failed to get arg \'fragile\'")?
				|| undo_on_err,
			dry: m
				.try_contains_id("dry")
				.context("Failed to get argument \'dry\'")?
				|| undo_on_err,
			dirs: m
				.try_contains_id("dirs")
				.context("Failed to get argument \'dirs'")?,
			suffix: m
				.try_get_one("suffix")
				.context("Failed to get argument \'suffix\'")?
				.cloned(),
			prefix: m
				.try_get_one("prefix")
				.context("Failed to get argument \'prefix\'")?
				.cloned(),
			output_dir: m
				.try_get_one("output-dir")
				.context("Failed to get argument  \'output-dir\'")?
				.cloned(),
			files,
			copy: m
				.try_contains_id("copy")
				.context("Failed to get field \'copy\'")?,
			on_conflict: m
				.try_get_one::<OnConflict>("on-conflict")
				.context("Failed to get argument \'on-conflict\'")?
				.unwrap_or(&OnConflict::Skip)
				.to_owned(),
			output_files,
			replace,
			replace_sets,
			config,
		})
	}
}
