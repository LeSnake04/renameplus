#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
pub mod args;
pub mod errors;
#[cfg(feature = "gui")]
pub mod gui;
pub mod input;
pub mod log;
pub mod parse_args;

pub use crate::input::read_input;
pub use crate::parse_args::ParseArgs;

use clap::ArgMatches;
use clap_logger::ClapInitLogger;
pub use clap_logger::{debug, error, info, trace, warn};
pub use miette::{miette, Result, WrapErr};
// For re-trigger command!() on toml changes
const _: &str = include_str!("../Cargo.toml");

#[macro_export]
macro_rules! use_err {
	() => {
		#[allow(unused_imports)]
		use $crate::{miette, or_wrap_err, unwrap_or_print, wrap_err, Result, WrapErr};
	};
}
#[macro_export]
macro_rules! wrap_err {
	($res: expr, $($msg: tt)*) => {
		::miette::Context::wrap_err(
			::miette::IntoDiagnostic::into_diagnostic($res),
			(format!($($msg)*))
		)
	};
}

#[macro_export]
macro_rules! or_wrap_err {
	($opt: expr, $($msg: tt)*) => ($opt.ok_or_else(|| ::miette::miette!($($msg)*)));
}

#[macro_export]
macro_rules! unwrap_or_print {
	($res: expr, $op: expr, $($msg: tt)*) => {
		match $res {
			Ok(r) => r,
			Err(e) => {
				::clap_logger::error!($($msg)+);
				$op;
			}
		}
	};
	($res: expr, $op: expr) => {
		match $res {
			Ok(r) => r,
			Err(e) => {
				::clap_logger::error!("{}", e);
				$op
			}
		}
	};
	($res: expr, $op: block, $($msg: tt)*) => {
		match $res {
			Ok(r) => r,
			Err(e) => {
				::clap_logger::error!($($msg)+);
				$op
			}
		}
	};
	($res: expr, $op: block) => {
		match $res {
			Ok(r) => r,
			Err(e) => {
				::clap_logger::error!("{}", e);
				$op;
			}
		}
	};
}

fn main() -> Result<()> {
	let m: ArgMatches = args::matches();
	log::log(wrap_err!(m.get_loglevel(), "Failed to init logger")?)?;
	ParseArgs::try_from(m)?.parse()?;
	Ok(())
}
