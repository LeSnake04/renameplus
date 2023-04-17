use std::{
	error::Error,
	fmt::{Debug, Display},
	ops::AddAssign,
};

use log::error;

use crate::into_none_if;

pub type ErrorLogAnyhow<T> = ErrorLog<T, anyhow::Error>;
// #[derive(Debug)]
pub struct ErrorLog<T, E = Box<dyn Error>> {
	errors: Vec<E>,
	ok: Option<T>,
	display_mode: PrintMode,
	print_fn: Box<dyn Fn(String)>,
	join: Option<String>,
}

impl<T, E> Default for ErrorLog<T, E> {
	fn default() -> Self {
		Self {
			ok: None,
			errors: Vec::new(),
			display_mode: PrintMode::default(),
			print_fn: Box::new(|e| println!("{e}")),
			join: None,
		}
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrintMode {
	#[default]
	/// Uses `{}`
	Normal,
	/// Uses `{:?}`
	Debug,
	/// Uses `{:#?}`
	PrettyDebug,
}

impl<T, E> ErrorLog<T, E> {
	pub fn append_errors<U>(&mut self, other: &mut ErrorLog<U, E>) -> &mut Self {
		self.errors.append(&mut other.errors);
		self
	}
	pub fn discard_err(self) -> Option<T> {
		self.ok
	}
	/// get Vector of errors
	pub fn errors(&self) -> &Vec<E> {
		&self.errors
	}
	/// Get mutable Vector of Errors
	pub fn errors_mut(&mut self) -> &mut Vec<E> {
		&mut self.errors
	}
	pub fn map<U>(self, fun: impl FnOnce(Self) -> ErrorLog<U, E>) -> ErrorLog<U, E> {
		fun(self)
	}
	pub fn map_errors<F>(self, fun: impl FnOnce(Vec<E>) -> Vec<F>) -> ErrorLog<T, F> {
		let mut out = ErrorLog::new();
		*out.errors_mut() = fun(self.errors);
		out
	}
	pub fn new() -> Self {
		Self::default()
	}
	pub fn ok(&self) -> &Option<T> {
		&self.ok
	}
	pub fn ok_mut(&mut self) -> &mut Option<T> {
		&mut self.ok
	}
	pub fn take_ok(&mut self) -> Option<T> {
		self.ok.take()
	}
	pub fn prepend_errors<U>(&mut self, other: &mut ErrorLog<U, E>) -> &mut Self {
		other.append_errors(self);
		self.append_errors(other);
		self
	}
	pub fn print_fn(&mut self, fun: impl Fn(String) + 'static) -> &mut Self {
		self.print_fn = Box::new(fun);
		self
	}
	pub fn push_err(&mut self, err: E) -> &mut Self {
		self.errors.push(err);
		self
	}
	pub fn push_result<U>(&mut self, res: Result<U, E>) -> Option<U> {
		match res {
			Ok(o) => Some(o),
			Err(err) => {
				self.errors.push(err);
				None
			}
		}
	}
	pub fn set_ok(&mut self, new: T) -> &mut Self {
		self.ok = Some(new);
		self
	}
}
impl<T, E: Display + Debug> ErrorLog<T, E> {
	pub fn join_to_string(&self, delimiter: impl Into<String>) -> Option<String> {
		let delimiter = delimiter.into();
		let mut out = String::from("");
		for err in &self.errors {
			out.push_str(&(self.as_string(err) + &delimiter))
		}
		into_none_if(out.is_empty(), out)
	}
	fn as_string(&self, err: &E) -> String {
		match self.display_mode {
			PrintMode::Normal => err.to_string(),
			PrintMode::Debug => format!("{err:?}"),
			PrintMode::PrettyDebug => format!("{err:#?}"),
		}
	}
	pub fn join_on_display(&mut self, delimiter: impl Into<String>) -> &mut Self {
		self.join = Some(delimiter.into());
		self
	}
	pub fn print_fn_log_error(&mut self) -> &mut Self {
		self.print_fn(|e| error!("{e}"))
	}
	/// Set how the errors should be printed:
	/// Normal: `{}`
	/// Debug: `{:?}`
	/// PrettyDebug: `{:#?}`
	pub fn display_mode(&mut self, mode: PrintMode) -> &mut Self {
		self.display_mode = mode;
		self
	}
	fn display(&self) {
		match self.join {
			None => {
				for err in &self.errors {
					(*self.print_fn)(self.as_string(err));
				}
			}
			Some(ref delimiter) => {
				if let Some(err) = self.join_to_string(delimiter) {
					(*self.print_fn)(err)
				}
			}
		}
	}
	pub fn get_display_mode(&self) -> &PrintMode {
		&self.display_mode
	}
	pub fn unwrap_or_display(self) -> Option<T> {
		self.display();
		self.ok
	}
	pub fn unwrap_or_display_and(self, or: T) -> T {
		self.display();
		self.ok.unwrap_or(or)
	}
	pub fn unwrap_or_display_and_default(self) -> T
	where
		T: Default,
	{
		self.display();
		self.ok.unwrap_or_default()
	}
	pub fn unwrap_or_display_and_else(self, run: impl FnOnce() -> T) -> T {
		self.display();
		self.ok.unwrap_or_else(run)
	}
}
impl<T> ErrorLog<T> {
	pub fn push_err_box(&mut self, err: impl Error + 'static) -> &mut Self {
		self.errors.push(Box::new(err));
		self
	}
	pub fn push_result_box<U, F: Error + 'static>(&mut self, res: Result<U, F>) -> Option<U> {
		match res {
			Ok(o) => Some(o),
			Err(err) => {
				self.errors.push(Box::new(err));
				None
			}
		}
	}
}
// impl<T: Sized> Deref for Recoverable<T> {
// 	type Target = Option<T>;
// 	fn deref(&self) -> &Self::Target {
// 		&self.ok
// 	}
// }
// impl<T> DerefMut for Recoverable<T> {
// 	fn deref_mut(&mut self) -> &mut Self::Target {
// 		&mut self.ok
// 	}
// }
impl<T> IntoIterator for ErrorLog<T> {
	type Item = T;
	type IntoIter = std::option::IntoIter<T>;
	fn into_iter(self) -> Self::IntoIter {
		self.ok.into_iter()
	}
}
impl<T, E: Debug + Display + 'static> AddAssign<E> for ErrorLog<T, E> {
	fn add_assign(&mut self, rhs: E) {
		self.push_err(rhs);
	}
}
impl<T, U, E: Debug + Display + 'static> AddAssign<Result<U, E>> for ErrorLog<T, E> {
	fn add_assign(&mut self, rhs: Result<U, E>) {
		self.push_result(rhs);
	}
}
