use std::path::PathBuf;

use clap::{
	builder::EnumValueParser, builder::PossibleValue, command, value_parser, Arg, ArgAction,
	ArgMatches, ValueHint,
};

pub fn matches() -> ArgMatches {
	command!()
		.arg(
			Arg::new("prefix")
				.long("prefix")
				.short('p')
				.value_parser(value_parser!(String))
				.value_name("PREFIX")
				.required(false)
				.help_heading("SIMPLE")
				.value_hint(ValueHint::Other)
				.help("Prefix to be added to the file"),
		)
		.arg(
			Arg::new("file")
				.value_parser(value_parser!(PathBuf))
				.value_name("FILE")
				.value_hint(ValueHint::AnyPath)
				.required(true)
				.action(ArgAction::Append)
				.help("File(s)  to be renamed"),
		)
		.arg(
			Arg::new("dry")
				.long("dry")
				.short('d')
				.help_heading("GENERAL")
				.action(ArgAction::SetTrue)
				.help("Dont perfrom the operations"),
		)
		.arg(
			Arg::new("dirs")
				.long("dirs")
				.short('r')
				.help_heading("GENERAL")
				.action(ArgAction::SetTrue)
				.help("Allow renaming of directories"),
		)
		.arg(
			Arg::new("copy")
				.long("copy")
				.short('c')
				.help_heading("GENERAL")
				.action(ArgAction::SetTrue)
				.help("Copy files instead of moving them"),
		)
		.arg(
			Arg::new("suffix")
				.long("suffix")
				.short('s')
				.value_name("SUFFIX")
				.value_hint(ValueHint::Other)
				.help_heading("SIMPLE")
				.help("Attach text to files"),
		)
		.arg(
			Arg::new("fragile")
				.long("fragile")
				.short('f')
				.action(ArgAction::SetTrue)
				.help("Crash as soon as a error occurs"),
		)
		.arg(
			Arg::new("undo-on-err")
				.long("undo-on-err")
				.short('u')
				.action(ArgAction::SetTrue)
				.conflicts_with("dry")
				.help_heading("GENERAL")
				.help("Undo previos actions if error ocours (implies fragile)"),
		)
		.arg(
			Arg::new("on-conflict")
				.long("on-conflict")
				.short('C')
				.help("What to do when target already exist")
				.value_parser(EnumValueParser::<OnConflict>::new())
				.value_hint(ValueHint::Other)
				.default_value("skip"),
		)
		.arg(
			Arg::new("output-dir")
				.value_parser(value_parser!(PathBuf))
				.long("output-dir")
				.short('o')
				.help_heading("GENERAL")
				.value_name("DIRECTORY")
				.value_hint(ValueHint::DirPath)
				.help("Move files to this directory"),
		)
		.arg(
			Arg::new("output-files")
				.value_parser(value_parser!(PathBuf))
				.long("output-files")
				.short('O')
				.action(ArgAction::Append)
				.value_hint(ValueHint::AnyPath)
				.help("Files to output files"),
		)
		.get_matches()
}

#[derive(Debug, Clone)]
pub enum OnConflict {
	Overwrite,
	Skip,
	Ask,
}

impl Default for OnConflict {
	fn default() -> Self {
		Self::Skip
	}
}

impl clap::ValueEnum for OnConflict {
	fn to_possible_value<'a>(&self) -> Option<PossibleValue> {
		Some(match self {
			Self::Overwrite => PossibleValue::new("overwrite").help("Overwrite the file"),
			Self::Skip => PossibleValue::new("skip").help("Skip file"),
			Self::Ask => PossibleValue::new("ask").help("Ask every time"),
		})
	}

	fn value_variants<'a>() -> &'a [Self] {
		&[Self::Skip, Self::Ask, Self::Overwrite]
	}
}
