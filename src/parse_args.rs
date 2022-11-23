use std::ffi::OsString;
use std::fs::{copy, rename};
use std::path::{Path, PathBuf};

use clap::ArgMatches;
use clap_logger::{debug, error, info, warn};
use miette::ensure;

use crate::args::OnConflict;
crate::use_err!();

pub struct ParseArgs {
	undo_on_err: bool,
	dry: bool,
	dirs: bool,
	files: Vec<PathBuf>,
	prefix: Option<String>,
	suffix: Option<String>,
	copy: bool,
	on_conflict: OnConflict,
	fragile: bool,
	output_dir: Option<PathBuf>,
	output_files: Option<Vec<PathBuf>>,
}

#[derive(Debug, Clone)]
pub struct ParsePathOut {
	original: PathBuf,
	new_path: Option<PathBuf>,
}

impl TryFrom<ArgMatches> for ParseArgs {
	type Error = miette::Error;

	fn try_from(m: clap::ArgMatches) -> Result<Self> {
		debug!("Parsing input args");
		let files: Vec<PathBuf> = or_wrap_err!(
			wrap_err!(
				m.try_get_many::<PathBuf>("file"),
				"Failed to get argument 'file'"
			)?,
			"Failed to get files"
		)
		.map(move |v| v.cloned().collect())?;
		let output_files: Option<Vec<PathBuf>> = wrap_err!(
			m.try_get_many::<PathBuf>("output-files"),
			"failed to get argument \'output-files\'",
		)?
		.map(move |v| v.cloned().collect());
		if let Some(ref o) = output_files {
			ensure!(o.is_empty(), "Need at least one output file");
			let output_file_last = or_wrap_err!(o.last(), "output_files is empty")?;
			if o.len() < files.len() && !output_file_last.exists() {
				Err(miette!("Last entry of array does not exist"))?;
				ensure!(output_file_last.is_dir(), "Last entry isn't a directory");
			}
		}
		let undo_on_err: bool = wrap_err!(
			m.try_contains_id("undo-on-err"),
			"Failed to get argument \'undo-on-errr\'"
		)?;
		Ok(Self {
			undo_on_err,
			fragile: wrap_err!(
				m.try_contains_id("fragile"),
				"failed to get arg \'fragile\'"
			)? || undo_on_err,
			dry: wrap_err!(m.try_contains_id("dry"), "Failed to get argument \'dry\'")?
				|| undo_on_err,
			dirs: wrap_err!(m.try_contains_id("dirs"), "Failed to get argument \'dirs'")?,
			suffix: wrap_err!(m.try_get_one("suffix"), "Failed to get argument \'suffix\'")?
				.cloned(),
			prefix: wrap_err!(m.try_get_one("prefix"), "Failed to get argument \'prefix\'")?
				.cloned(),
			output_dir: wrap_err!(
				m.try_get_one("output-dir"),
				"Failed to get argument  \'output-dir\'"
			)?
			.cloned(),
			files,
			copy: wrap_err!(m.try_contains_id("copy"), "Failed to get field \'copy\'")?,
			on_conflict: wrap_err!(
				m.try_get_one::<OnConflict>("on-conflict"),
				"Failed to get argument \'on-conflict\'"
			)?
			.unwrap_or(&OnConflict::Skip)
			.to_owned(),
			output_files,
		})
	}
}

impl ParseArgs {
	fn get_parent(&self, file: &Path) -> Result<PathBuf> {
		match &self.output_dir {
			// get parent if no output dir set.
			None => or_wrap_err!(
				file.parent().map(|f| f.to_owned()),
				"Failed to get parent of {}",
				file.display()
			),
			// return output dir if set.
			o => or_wrap_err!(o.to_owned(), "Failed to get output dir"),
		}
	}
	pub fn parse(self) -> Result<()> {
		let mut history: Vec<ParsePathOut> = vec![];
		let mut out: Result<()> = Ok(());

		for file in &self.files {
			let curr_out = self.parse_path(file);
			match (curr_out, self.fragile, self.undo_on_err) {
				// Cancel if error occured and --fragile set.
				(Err(e), true, _) => {
					out = Err(e);
					break;
				}
				// Print error if --fragile not set.
				(Err(e), false, _) => error!("{:?}", e),
				// Push result to $history if moved and --und_on_err set.
				(Ok(r), _, true) => {
					r.new_path.is_some().then(|| history.push(r));
				}
				_ => (),
			}
		}
		if out.is_err() && self.undo_on_err {
			for out in history {
				let new_path = match or_wrap_err!(out.new_path, "Path not set") {
					// Print error if
					Err(f) => {
						error!("{f}");
						continue;
					}
					//
					Ok(p) => p,
				};
				// Perfom file Operations
				// Print error on fail
				if let Err(e) = match self.copy {
					// Remove file
					true => wrap_err!(std::fs::remove_file(new_path), "Failed to remove file"),
					// Rename file
					false => wrap_err!(rename(new_path, out.original), "Failed to rename file"),
				} {
					// Print error
					error!("{}", e)
				}
			}
		}
		Ok(())
	}
	fn parse_path(&self, file: &PathBuf) -> Result<ParsePathOut> {
		if !wrap_err!(file.try_exists(), "Error parsing file")? {
			Err(miette::miette!(format!(
				"File {} does not exist",
				file.display()
			)))?;
		}
		let original: PathBuf = file.clone();
		let path: Result<PathBuf> = {
			or_wrap_err!(
				match file.file_name() {
					None => match (file.parent(), self.dirs) {
						(None, false) => None,
						// Return early if $file is a folder and --dirs not set.
						(Some(p), false) => {
							warn!(
								"Skipped {} bevause it is a Directory. Use \'-r\' or \'--dirs\'",
								p.display()
							);
							return Ok(ParsePathOut {
								original: file.clone(),
								new_path: None,
							});
						}
						(p, true) => p.map(|f| f.to_path_buf()),
					},
					o => o.map(|f| f.into()),
				},
				"Failed to parse path {}",
				file.display()
			)
		};
		let mut new_name: OsString = {
			let out = path?;
			out.file_stem()
				.unwrap_or(or_wrap_err!(
					out.file_name(),
					"Failed to get file name or stem"
				)?)
				.to_owned()
		};

		if let Some(p) = &self.prefix {
			let mut out: OsString = p.into();
			out.push(&new_name);
			new_name = out;
		}

		if let Some(s) = &self.suffix {
			new_name.push(s);
		}

		let parent = self
			.get_parent(file)
			.wrap_err("Failed to get parent of path")?;
		let mut new_path: PathBuf = {
			let mut out: PathBuf = parent;
			out.push(&new_name);
			out
		};
		if new_path.exists() {
			let mut on_conflict: OnConflict = self.on_conflict.clone();
			loop {
				match on_conflict {
					OnConflict::Ask => match self.conflict_ask(&new_path) {
						Ok((Some(p), o)) => {
							on_conflict = o;
							new_path = p;
							continue;
						}
						Ok((None, o)) => {
							on_conflict = o;
							continue;
						}
						Err(e) => {
							Err(e)?;
						}
					},
					OnConflict::Skip => {
						return Ok(ParsePathOut {
							original: file.to_owned(),
							new_path: None,
						})
					}
					_ => (),
				}
				break;
			}
		}
		info!("{} -> {}", file.display(), new_path.display());
		match (self.dry, self.copy) {
			(false, true) => {
				wrap_err!(
					copy(file, &new_path),
					"Failed to copy file {}",
					file.display()
				)?;
			}
			(false, false) => wrap_err!(
				rename(file, &new_path),
				"Failed to rename file {}",
				file.display()
			)?,
			(true, _) => info!(
				"{} Skipped because of \'--dry\' or \'--copy \'.",
				file.display()
			),
		}
		Ok(ParsePathOut {
			original,
			new_path: Some(new_path),
		})
	}
	pub fn verify_output_dir(&self) -> Result<&Self> {
		if let Some(o) = &self.output_dir {
			if !o.exists() {
				Err(miette!(format!(
					"Output dir {} doesn\'t exist",
					o.display()
				)))?;
			}
		}
		Ok(self)
	}
}
