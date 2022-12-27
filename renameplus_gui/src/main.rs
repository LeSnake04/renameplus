#![warn(clippy::all)]

use anyhow::{Context, Result};
use flexi_logger::Logger;
use iced::{Application, Settings};
use log::warn;

pub use crate::gui::RenamePlusGui;

pub mod file;
pub mod gui;
pub mod helper;
pub mod replace;
pub mod run;
pub mod update;

fn main() -> Result<()> {
	Logger::try_with_env_or_str("renameplus_gui=debug, off")
		.context("Failed to parse logger config")?
		.start()
		.context("Failed to init logger")?;
	// force Xwayland for file drag drog support.
	#[cfg(target_os = "linux")]
	std::env::set_var("WINIT_UNIX_BACKEND", "x11");
	RenamePlusGui::run(Settings::with_flags(())).context("Faild to start gui")?;
	Ok(())
}
