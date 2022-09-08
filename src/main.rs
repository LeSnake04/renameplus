#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
mod args;

use std::{fs::copy, path::PathBuf};

use clap::ArgMatches;
use clap_logger::{info, ClapInitLogger};
use_err!();

#[macro_export]
macro_rules! use_err {
	() => {
		#[allow(unused_imports)]
		use miette::{miette, IntoDiagnostic, Result, WrapErr};
	};
}

#[macro_export]
macro_rules! err {
	($err: expr, $msg: literal) => {{
		$err.into_diagnostic().wrap_err($msg)
	}};
}

fn main() -> Result<()> {
	let m: ArgMatches = args::matches();
	err!(m.to_owned().init_logger(), "Failed to init logger!")?;
	// .wrap_err("Failed to init logger")?;
	let prefix: Option<&String> = m.get_one::<String>("prefix");
	let files: Vec<&PathBuf> = m
		.get_many::<PathBuf>("file")
		.ok_or_else(|| miette!(""))
		.wrap_err("failed to read files")?
		.collect();

	for path in files {
		let get_path = || -> Result<PathBuf> {
			match path.file_name() {
				None => match (path.parent(), m.contains_id("allow_dirs")) {
					(None, false) => None,
					(p, true) => p.map(|f| f.to_path_buf()),
					(p, false) => p.map(|r| r.to_owned()),
				},
				o => o.map(|f| f.into()),
			}
			.ok_or_else(|| miette!("Failed to get path {}", path.display()))
			.wrap_err("Failed to get Error")
		};
		let get_parent = |file: PathBuf| -> Result<PathBuf> {
			file.parent()
				.map(|f| f.to_owned())
				.ok_or_else(|| miette!("Failed to get parent of {}", file.display()))
		};
		let mut new_name: String = get_path()?.to_string_lossy().into();

		if let Some(p) = prefix {
			let mut p = p.clone();
			p.push_str(&new_name);
			new_name = p.to_owned();
		}
		let new_path: PathBuf = {
			let mut out: PathBuf = get_parent(path.to_owned())?;
			out.push(&new_name);
			out
		};
		if m.is_present("dry") {
			println!("{}", new_name);
		} else {
			info!("Copying {} to {}", path.display(), new_path.display());
			if !err!(m.try_contains_id("dry"), "Failed to get argument 'pass'")? {
				err!(copy(path, new_path), "Failed to copy file")?;
				// remove_file(file);
			}
		}
	}
	Ok(())
}
