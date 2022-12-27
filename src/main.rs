#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

pub mod args;
pub mod config;
pub mod input;
pub mod new_rename;
pub mod rename;

pub use crate::args::OnConflict;
pub use crate::config::Config;
pub use crate::input::read_input;
pub use crate::rename::Rename;

use anyhow::{Context, Result};
use clap::ArgMatches;
use flexi_logger::Logger;
// For re-trigger command!() on toml changes
const _: &str = include_str!("../Cargo.toml");

fn main() -> Result<()> {
	Logger::try_with_env_or_str("debug")
		.context("Failed to init logger")?
		.start()?;
	let conf = Config::read()?;
	let m: ArgMatches = args::matches();
	Rename::new(m, conf)?.rename()?;
	Ok(())
}
