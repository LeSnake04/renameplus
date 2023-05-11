#![warn(clippy::all)]
#![warn(clippy::unwrap_used)]

use anyhow::{Context, Result};
use flexi_logger::Logger;
use iced::{Application, Settings};
use log::warn;

pub use iced::widget::column as col;

pub use crate::gui::RenamePlusGui;
pub use crate::presets::*;
pub use crate::update::*;
pub use crate::widgets::*;

pub mod gui;
pub mod helper;
pub mod presets;
pub mod run;
pub mod update;
mod view;
pub mod widgets;

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
