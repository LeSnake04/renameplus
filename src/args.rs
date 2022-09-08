use std::path::PathBuf;

use crate::error::*;
use clap::{value_parser, Arg, ArgMatches, Command};
use clap_logger::{ClapLoglevelArg, LevelFilter};
use miette::WrapErr;

pub fn matches() -> Result<ArgMatches> {
	Command::new("renameplus")
		.author("LeSnake <dev.lesnake@posteo.de>")
		.add_loglevel_arg(LevelFilter::Warn)
		.arg(
			Arg::new("prefix")
				.value_parser(value_parser!(String))
				.short('p')
				.value_name("PREFIX")
				.required(false),
		)
		.arg(
			Arg::new("file")
				.value_parser(value_parser!(PathBuf))
				.multiple_values(true)
				.value_name("FILE")
				.required(true),
		)
		.arg(Arg::new("dry").long("dry"))
		.arg(Arg::new("allow-dirs").short('r'))
		.arg(Arg::new("copy").short('c'))
		.try_get_matches()
		.map_err(|e| Error::ArgParseError(e))
		.wrap_err("Failed to parse Arguments")
}
