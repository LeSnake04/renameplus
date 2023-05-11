use bstr::ByteSlice;
use error_log::{return_ok, try_add, ErrorLogAnyhow};
use format as f;
use log::{error, info, warn};
use snake_helper::{unwrap_or_print_err, unwrap_some_or};
use std::ffi::OsString;
use std::fs::{copy, rename};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::args::OnConflict;
use crate::config::Config;

#[derive(Debug, Clone, Default)]
pub struct Rename {
	pub undo_on_err: bool,
	pub dry: bool,
	pub dirs: bool,
	pub files: Vec<PathBuf>,
	pub prefix: Option<String>,
	pub suffix: Option<String>,
	pub copy: bool,
	pub on_conflict: OnConflict,
	pub fragile: bool,
	pub output_dir: Option<PathBuf>,
	pub output_files: Option<Vec<PathBuf>>,
	pub replace: Vec<(String, String)>,
	pub replace_sets: Vec<String>,
	pub config: Config,
}

#[derive(Debug, Clone)]
pub struct RenameOut {
	original: PathBuf,
	new_path: Option<PathBuf>,
}

impl Rename {
	pub fn get_new_path(&self, file: &Path) -> Result<Option<PathBuf>> {
		if !file.try_exists().context("Error parsing file")? {
			Err(anyhow!("File {} does not exist", file.display()))?;
		}
		let path: Result<PathBuf> = {
			match file.file_name() {
				None => match (file.parent(), self.dirs) {
					(None, false) => None,
					// Cancel if file is a folder and --dirs not set.
					(Some(p), false) => {
						warn!(
							"Skipped {} bevause it is a Directory. Use \'-r\' or \'--dirs\' to rename directories",
							p.display()
						);
						return Ok(None);
					}
					(p, true) => p.map(|f| f.to_path_buf()),
				},
				o => o.map(|f| f.into()),
			}
			.context(format!("Failed to parse path {}", file.display()))
		};
		let (mut new_name, ext): (OsString, OsString) = {
			let out = path?;
			(
				out.file_stem()
					.unwrap_or(out.file_name().context("Failed to get file name or stem")?)
					.to_owned(),
				out.extension()
					.map(|e| {
						let mut out = OsString::from(".");
						out.push(e);
						out
					})
					.unwrap_or(OsString::from("")),
			)
		};

		if let Some(p) = &self.prefix {
			let mut out: OsString = p.into();
			out.push(&new_name);
			new_name = out;
		}

		if let Some(s) = &self.suffix {
			new_name.push(s);
		}

		for (search, replace) in &self.replace {
			do_replace(&mut new_name, search, replace)
		}
		for set_name in &self.replace_sets {
			let set = &self
				.config
				.sets
				.get(set_name)
				.context(format!("Set {set_name} not found"))?;
			for search in &set.set.search {
				do_replace(&mut new_name, search, &set.set.replace)
			}
		}
		let parent = self
			.get_parent(file)
			.context("Failed to get parent of path")?;
		let mut out: PathBuf = parent;
		new_name.push(ext);
		out.push(&new_name);
		Ok(Some(out))
	}
	fn get_parent(&self, file: &Path) -> Result<PathBuf> {
		match &self.output_dir {
			// get parent if no output dir set.
			None => file
				.parent()
				.map(|f| f.to_owned())
				.context(format!("Failed to get parent of {}", file.display())),
			o => o.to_owned().context("Failed to get output dir"),
		}
	}
	pub fn preview(&self) -> Result<Vec<(PathBuf, Option<PathBuf>)>> {
		let mut out: Vec<(PathBuf, Option<PathBuf>)> = vec![];
		for file in &self.files {
			let new = unwrap_or_print_err!(self.get_new_path(file), continue);
			out.push((file.clone(), new))
		}
		Ok(out)
	}
	pub fn push_replace(&mut self, search: impl Into<String>, replace: impl Into<String>) {
		self.replace.push((search.into(), replace.into()))
	}
	pub fn rename(&self) -> ErrorLogAnyhow<()> {
		let mut history: Vec<RenameOut> = vec![];
		let mut err = ErrorLogAnyhow::new();
		if let Some(preview) = err.push_result(self.preview()) {
			for (file, new_path) in preview {
				let new_path = unwrap_some_or!(new_path, continue);
				let curr_out = self.rename_file(&file, new_path);
				match (curr_out, self.fragile, self.undo_on_err) {
					// Cancel if error occured and --fragile set.
					(Err(e), true, _) => {
						try_add!(Err(e), err);
					}
					// Print error if --fragile not set.
					(Err(e), false, _) => err += e,
					// Push result to $history if moved and --und_on_err set.
					(Ok(r), _, true) => {
						r.new_path.is_some().then(|| history.push(r));
					}
					_ => (),
				}
			}
		}
		if self.undo_on_err && !err.entries().is_empty() {
			for entry in history {
				let new_path = match entry.new_path.context("Path not set") {
					Err(f) => {
						err.push_err(f);
						continue;
					}
					//
					Ok(p) => p,
				};
				if let Err(e) = match self.copy {
					true => std::fs::remove_file(new_path).context("Failed to remove file"),
					false => {
						std::fs::rename(new_path, entry.original).context("Failed to rename file")
					}
				} {
					error!("{}", e)
				}
			}
		}
		return_ok!((), err);
	}
	fn rename_file(&self, file: &PathBuf, mut new_path: PathBuf) -> Result<RenameOut> {
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
						return Ok(RenameOut {
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
				copy(file, &new_path).context(f!("Failed to copy file {}", file.display()))?;
			}
			(false, false) => {
				rename(file, &new_path).context(f!("Failed to rename file {}", file.display()))?
			}
			(true, _) => info!(
				"{} Skipped because of \'--dry\' or \'--copy \'.",
				file.display()
			),
		}
		Ok(RenameOut {
			original: file.to_owned(),
			new_path: Some(new_path),
		})
	}
	pub fn verify_output_dir(&self) -> Result<&Self> {
		if let Some(o) = &self.output_dir {
			if !o.exists() {
				Err(anyhow!("Output dir {} doesn\'t exist", o.display()))?;
			}
		}
		Ok(self)
	}
}
fn do_replace(str: &mut OsString, search: &str, replace: &str) {
	*str = {
		#[cfg(all(not(windows), not(unix)))]
		{
			error!("search and replace not supported for your os");
			break;
		}
		#[cfg(unix)]
		{
			use std::os::unix::ffi::{OsStrExt, OsStringExt};
			OsString::from_vec(str.as_bytes().replace(search, replace))
		}
		#[cfg(windows)]
		{
			use std::os::windows::ffi::{OsStrExt, OsStringExt};
			OsString::from_vec(str.as_bytes().replace(search, replace))
		}
	}
}
