use std::path::PathBuf;

use crate::use_err;
use clap::{value_parser, Arg, ArgMatches, Command};
use clap_logger::{ClapLoglevelArg, LevelFilter};
use_err!();

pub fn matches() -> ArgMatches {
	Command::new("autorename")
		.about("Tool to rename files")
		.arg_required_else_help(true)
		.author("LeSnake <dev.lesnake@posteo.de>")
		.add_loglevel_arg(LevelFilter::Warn)
		.arg(
			Arg::new("prefix")
				.long("prefix")
				.short('p')
				.value_parser(value_parser!(String))
				.value_name("PREFIX")
				.required(false)
				.help_heading("SIMPLE")
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
				.short('d')
				.help("Dont perfrom the operations"),
		)
		.arg(
			Arg::new("dirs")
				.long("dirs")
				.short('r')
				.help("Allow renaming of directories"),
		)
		.arg(
			Arg::new("copy")
				.long("copy")
				.short('c')
				.help("Copy files instead of moving them"),
		)
		.arg(
			Arg::new("suffix")
				.long("suffix")
				.short('s')
				.value_name("SUFFIX")
				.help_heading("SIMPLE")
				.help("Attach text to files"),
		)
		.arg(
			Arg::new("fragile")
				.long("fragile")
				.short('f')
				.help("Crash as soon as a error occurs"),
		)
		.get_matches()
}
