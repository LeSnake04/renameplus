#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
mod args;

use std::{
	fs::{copy, remove_file},
	path::PathBuf,
};

use clap::ArgMatches;
use clap_logger::{debug, error, info, warn, ClapInitLogger};
use miette::{miette, IntoDiagnostic, Result, WrapErr};
// use miette::miette;

#[macro_export]
macro_rules! use_err {
	() => {
		#[allow(unused_imports)]
		use miette::{miette, IntoDiagnostic, Result, WrapErr};
		#[allow(unused_imports)]
		use $crate::{err, or_err};
	};
}
#[macro_export]
macro_rules! err {
		($res: expr, $msg: literal) => ($res.into_diagnostic().wrap_err($msg));
		($res: expr, $($msg: tt)*) => ($res.into_diagnostic().wrap_err(format!($($msg)*)));
}

#[macro_export]
macro_rules! or_err {
		($opt: expr, $msg: literal) => ($opt.ok_or_else(|| miette!($msg)));
		($opt: expr, $($msg: tt)*) => ($opt.ok_or_else(|| miette!($($msg)*)));
}

fn main() -> Result<()> {
	let m: ArgMatches = args::matches();
	err!(m.to_owned().init_logger(), "Failed to init logger")?;
	let prefix: Option<&String> = m.get_one::<String>("prefix");
	let files: Vec<&PathBuf> = or_err!(
		err!(
			m.try_get_many::<PathBuf>("file"),
			"Failed to get argument 'file'"
		)?,
		"Failed to get files"
	)?
	.collect();
	let dry: bool = err!(m.try_contains_id("dry"), "Failed to get argument \'dry\'")?;
	let allow_dirs: bool = err!(m.try_contains_id("dirs"), "Failed to get argument 'dirs'")?;
	let fragile: bool = err!(
		m.try_contains_id("fragile"),
		"failed to get arg \'fragile\'"
	)?;
	let suffix: Option<&String> =
		err!(m.try_get_one("suffix"), "Failed to get argument \'suffix\'")?;
	for file in files {
		let out = parse_path(file, &m, dry, allow_dirs, prefix, suffix);
		match (&out, fragile) {
			(Err(_), true) => out?,
			(Err(e), false) => error!("{:?}", e),
			(Ok(_), _) => (),
		}
	}
	Ok(())
}

fn parse_path(
	file: &PathBuf,
	m: &ArgMatches,
	dry: bool,
	allow_dirs: bool,
	prefix: Option<&String>,
	suffix: Option<&String>,
) -> Result<()> {
	if !file.exists() {
		Err(miette!(format!("File {} does not exist", file.display())))?;
	}
	let path: Result<PathBuf> =
		{
			or_err!(
				match file.file_name() {
					None =>
						match (file.parent(), allow_dirs,) {
							(None, false) => None,
							(Some(p), true) => {
								warn!("Skipped {} bevause it is a Directory. Use \'-r\' or \'--dirs\'", p.display());
								return Ok(());
							}
							(p, true) => p.map(|f| f.to_path_buf()),
							(p, false) => p.map(|r| r.to_owned()),
						},
					o => o.map(|f| f.into()),
				},
				"Failed to parse path {}",
				file.display()
			)
		};
	let get_parent = |file: PathBuf| -> Result<PathBuf> {
		or_err!(
			file.parent().map(|f| f.to_owned()),
			"Failed to get parent of {}",
			file.display()
		)
	};
	let mut new_name: String = path?.to_string_lossy().into();

	if let Some(p) = prefix {
		let mut out: String = p.to_owned();
		out.push_str(&new_name);
		new_name = out;
	}

	if let Some(s) = suffix {
		new_name.push_str(s);
	}

	let new_path: PathBuf = {
		let mut out: PathBuf =
			get_parent(file.to_owned()).wrap_err("Failed to get parent of path")?;
		out.push(&new_name);
		out
	};
	info!("{} -> {}", file.display(), new_path.display());
	if !dry {
		err!(
			copy(file, new_path),
			"Failed to copy file {}",
			file.display()
		)?;
	} else {
		println!("dry");
		debug!("Skipped because of \'--dry\'.");
	}

	if !dry && !err!(m.try_contains_id("copy"), "Failed to get field \'copy\'")? {
		err!(
			remove_file(file),
			"Failed to remove file {}",
			file.display()
		)?;
	} else {
		info!(
			"{} Skipped because of \'--dry\' or \'--copy \'.",
			file.display()
		);
	}
	Ok(())
}
