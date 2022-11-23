use miette::Diagnostic;
use thiserror::Error;

pub type LogResult<T> = Result<T, LogError>;

#[derive(Debug, Diagnostic ,Error)]
pub enum LogError {
	#[error("Failed init logger")]
	InitFailed(#[from] log::SetLoggerError),
}
