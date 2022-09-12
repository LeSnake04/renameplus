#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
mod args;
pub mod parse;

use clap::ArgMatches;
use clap_logger::ClapInitLogger;
pub use clap_logger::{debug, error, info, trace, warn};
use miette::{miette, IntoDiagnostic, Result, WrapErr};
use parse::run_with_path;
use std::path::PathBuf;
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
	let ensure: bool = err!(
		m.try_contains_id("ensure"),
		"Failed to get argument \'ensure\'"
	)?;
	let dry: bool = err!(m.try_contains_id("dry"), "Failed to get argument \'dry\'")? || ensure;
	let allow_dirs: bool = err!(m.try_contains_id("dirs"), "Failed to get argument 'dirs'")?;
	let fragile: bool = err!(
		m.try_contains_id("fragile"),
		"failed to get arg \'fragile\'"
	)? || ensure;
	let suffix: Option<&String> =
		err!(m.try_get_one("suffix"), "Failed to get argument \'suffix\'")?;
	let output_dir: Option<&PathBuf> = err!(
		m.try_get_one("output-dir"),
		"Failed to get argument  \'output-dir\'"
	)?;
	if let Some(o) = output_dir {
		if !o.exists() {
			Err(miette!(format!(
				"Output dir {} doesn\'t exist",
				o.display()
			)))?;
		}
	}
	let files2 = match ensure {
		true => Some(files.clone()),
		false => None,
	};

	for file in files {
		let out = run_with_path(file, &m, dry, allow_dirs, prefix, suffix, output_dir);
		match (&out, fragile) {
			(Err(_), true) => out?,
			(Err(e), false) => error!("{:?}", e),
			(Ok(_), _) => (),
		}
	}
	if ensure {
		for file in or_err!(files2, "Failed to get files clone")? {
			let out = run_with_path(file, &m, !dry, allow_dirs, prefix, suffix, output_dir);
			match (&out, fragile) {
				(Err(_), true) => out?,
				(Err(e), false) => error!("{:?}", e),
				_ => (),
			}
		}
	}
	Ok(())
}
