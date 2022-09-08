#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
mod args;
mod error;
mod log;

use std::{
	fs::{copy, remove_file},
	path::{Ancestors, PathBuf},
};

use crate::error::*;
use clap::ArgMatches;
use clap_logger::ClapInitLogger;
use miette::{IntoDiagnostic, WrapErr};

fn main() -> Result<()> {
	let m: ArgMatches = args::matches()
		.into_diagnostic()
		.wrap_err("Failed get Arg matches")?;
	m.to_owned()
		.init_logger()
		.map_err(|e| Error::InitLoggerFailed)?;
	// .wrap_err("Failed to init logger")?;
	let prefix: Option<&String> = m.get_one::<String>("prefix");
	let files: Vec<&PathBuf> = m
		.get_many::<PathBuf>("file")
		.ok_or(Error::NoPathsSpecified)?
		.collect();

	for file in files {
		let get_path = || -> Result<PathBuf> {
			match file.file_name() {
				None => match (file.parent(), m.contains_id("allow_dirs")) {
					(None, false) => None,
					(p, true) => p.map(|f| f.to_path_buf()),
					(p, false) => p.map(|r| r.to_owned()),
				},
				o => o.map(|f| f.into()),
			}
			.ok_or(Error::ParentNotFound(file))
		};
		let get_parent = |file: PathBuf| -> Result<PathBuf> {
			file.parent()
				.map(|f| f.to_owned())
				.ok_or(Error::ParentNotFound(&file))
		};
		let mut new_name: String = get_path()?.to_string_lossy().into();

		if let Some(p) = prefix {
			let mut p = p.clone();
			p.push_str(&new_name);
			new_name = p.to_owned();
		}
		let new_path: PathBuf = {
			let mut out: PathBuf = get_parent(file.to_owned())?;
			out.push(&new_name);
			out
		};
		if m.is_present("dry") {
			println!("{}", new_name);
		} else {
			copy(file, new_path);
			if m.contains_id("pass") {
				// remove_file(file);
			}
			println!("{}", new_name);
		}
	}
	Ok(())
}
