use clap_logger::{debug, error, info};

use crate::args::OnConflict;
use crate::{use_err, ParseArgs};
use std::fs::rename;
use std::path::PathBuf;
use_err!();

pub fn read_input() -> Result<String> {
	use std::io::Write;
	wrap_err!(std::io::stdout().flush(), "Failed to flush stdout")?;
	let mut input: String = "".into();
	wrap_err!(
		std::io::stdin().read_line(&mut input),
		"Failed to read line"
	)?
	.to_string();
	Ok(input)
}

pub fn read_input_loop(limit: u8) -> Result<String> {
	let mut tries: u8 = 0;
	let mut res = read_input();
	while let Err(e) = res {
		if tries > limit {
			return Err(miette!("Limit reached"));
		}
		tries += 1;
		error!("{}", e);
		res = read_input();
	}
	res
}

impl ParseArgs {
	pub(crate) fn conflict_ask(&self, new_path: &PathBuf) -> Result<(Option<PathBuf>, OnConflict)> {
		let mut input: String;
		loop {
			print!(
				"Target file {} already exist\n[o]verwrite, [s]kip, [r]ename new, [R]ename original, [a]bort, s[u]ffix new,s[U]ffix original, [p]refix new,  [P]refix old: ",
				new_path.display()
			);
			input = unwrap_or_print!(read_input_loop(5), continue).trim().into();
			debug!("Input: {}", input);
			match input.as_str() {
				"o" => {
					info!("Overwriting file");
					return Ok((None, OnConflict::Overwrite));
				}
				"s" => return Ok((None, OnConflict::Skip)),
				"r" | "R" => {
					print!("Enter name for new folder: ");
					let new_name = unwrap_or_print!(read_input(), continue);
					let path: PathBuf = {
						let mut out = or_wrap_err!(new_path.clone().parent(), "No parent found.")?
							.to_path_buf();
						out.push(&new_name);
						out
					};
					if input == "R" {
						wrap_err!(rename(new_path, path), "failed to rename orginal")?;
						continue;
					} else {
						return Ok((Some(path), OnConflict::Ask));
					}
				}
				"p" | "P" => {
					print!("Enter prefix: ");
					let new_name = {
						let mut out = unwrap_or_print!(read_input(), continue);
						out.push_str(or_wrap_err!(
							or_wrap_err!(new_path.file_stem(), "Failed to get file name")?.to_str(),
							"Failed to convert to str"
						)?);
						out
					};
					let path: PathBuf = {
						let mut out =
							or_wrap_err!(new_path.clone().parent(), "Failed to get parent")?
								.to_path_buf();
						out.push(&new_name);
						out
					};
					if input == "P" {
						wrap_err!(rename(new_path, path), "failed to rename orginal")?;
					} else {
						return Ok((Some(path), OnConflict::Ask));
					}
				}
				"u" | "U" => {}
				i => error!("Invalid input: {:?}", i),
			}
		}
	}
}
