use std::{fmt::Display, path::PathBuf};

use miette::Diagnostic;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Diagnostic)]
pub enum Error {
	InitLoggerFailed,
	NoPathsSpecified,
	FileError(std::io::Error),
	// NoFileName,
	ArgParseError(clap::Error),
	ParentNotFound(&'static PathBuf),
	InputParentNotFound(&'static PathBuf),
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Error::InitLoggerFailed => "Starting logger failed".into(),
				Error::NoPathsSpecified => "No Paths Specified".into(),
				Error::FileError(e) => format!("{}", e),
				Error::ArgParseError(a) => format!("{}", a),
				Error::ParentNotFound(p) => format!("Parent of {} not found", p.display()),
				Error::InputParentNotFound(e) =>
					format!("Parent of input {} not found", e.display()),
			}
		)
	}
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
	fn from(e: std::io::Error) -> Self {
		Self::FileError(e)
	}
}
