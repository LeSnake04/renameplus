use std::path::PathBuf;

use crate::use_err;
use clap::{value_parser, Arg, ArgMatches, Command};
use clap_logger::{ClapLoglevelArg, LevelFilter};
use_err!();

// pub fn matches() -> Result<ArgMatches> {
pub fn matches() -> ArgMatches {
	// err!(
	Command::new("autorename")
		.about("Tool to rename files")
		.arg_required_else_help(true)
		.author("LeSnake <dev.lesnake@posteo.de>")
		.add_loglevel_arg(LevelFilter::Warn)
		.arg(
			Arg::new("prefix")
				.value_parser(value_parser!(String))
				.short('p')
				.value_name("PREFIX")
				.required(false)
				.help("Prefix to be added to the file"),
		)
		.arg(
			Arg::new("file")
				.value_parser(value_parser!(PathBuf))
				.multiple_values(true)
				.value_name("FILE")
				.required(true)
				.help("File(s)  to be renamed"),
		)
		.arg(
			Arg::new("dry")
				.long("dry")
				.help("Dont perfrom the operations"),
		)
		.arg(
			Arg::new("allow-dirs")
				.short('r')
				.help("Allow renaming of directories"),
		)
		.arg(
			Arg::new("copy")
				.short('c')
				.help("Copy files instead of moving them"),
		)
		.get_matches()
	// .try_get_matches(),
	// "Failed to get Arguments"
	// )
}
