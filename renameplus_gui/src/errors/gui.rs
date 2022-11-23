use miette::Diagnostic;
use thiserror::Error;

pub type GuiResult<T> = Result<T, GuiError>;

#[derive(Debug, Error, Diagnostic)]
pub enum GuiError {
	#[error("Failed to set notification level")]
	InvalidNotificationLevel(String),
}
