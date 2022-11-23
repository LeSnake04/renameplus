use crate::errors::log::LogResult as Result;
use clap_logger::LevelFilter;
use fern::colors::{Color, ColoredLevelConfig};
use owo_colors::OwoColorize;

pub fn log(loglevel: LevelFilter) -> Result<()> {
	let colors: ColoredLevelConfig = fern::colors::ColoredLevelConfig::new()
		.error(Color::BrightRed)
		.warn(Color::Yellow)
		.info(Color::BrightGreen)
		.debug(Color::Cyan)
		.trace(Color::BrightBlack);
	Ok(fern::Dispatch::new()
		.level(loglevel)
		.format(move |out, message, record| {
			out.finish(format_args!(
				"{} {}: {}",
				colors.color(record.level()),
				format!("[{}]", record.target()).fg_rgb::<80, 80, 80>(),
				message
			))
		})
		.chain(std::io::stdout())
		.apply()?)
}
