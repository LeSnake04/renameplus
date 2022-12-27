use anyhow::{anyhow, Context, Result};
use dialoguer::Select;
use log::{debug, error, info};
use snake_helper::unwrap_or_print_err;

use crate::args::OnConflict;
use crate::rename::Rename;

use std::fs::rename;
use std::path::PathBuf;

pub fn read_input() -> Result<String> {
	use std::io::Write;
	std::io::stdout()
		.flush()
		.context("Failed to flush stdout")?;
	let mut input: String = "".into();
	std::io::stdin()
		.read_line(&mut input)
		.context("Failed to read line")?
		.to_string();
	Ok(input)
}

pub fn read_input_loop(limit: u8) -> Result<String> {
	let mut tries: u8 = 0;
	let mut res = read_input();
	while let Err(e) = res {
		if tries > limit {
			return Err(anyhow!("Limit reached"));
		}
		tries += 1;
		error!("{}", e);
		res = read_input();
	}
	res
}

impl Rename {
	pub fn conflict_ask(&self, new_path: &PathBuf) -> Result<(Option<PathBuf>, OnConflict)> {
		loop {
			print!(
				"Target file {} already exist\n[o]verwrite, [s]kip, [r]ename new, [R]ename original, [a]bort, s[u]ffix new,s[U]ffix original, [p]refix new,  [P]refix old: ",
				new_path.display()
			);
			Select::new().items(&["o", "b", "c"]).default(0);
			let input: String = unwrap_or_print_err!(read_input_loop(5), continue)
				.trim()
				.into();
			debug!("Input: {}", input);
			match input.as_str() {
				"o" => {
					info!("Overwriting file");
					return Ok((None, OnConflict::Overwrite));
				}
				"s" => return Ok((None, OnConflict::Skip)),
				"r" | "R" => {
					print!("Enter name for new folder: ");
					let new_name = unwrap_or_print_err!(read_input(), continue);
					let path: PathBuf = {
						let mut out = new_path
							.clone()
							.parent()
							.context("No parent found.")?
							.to_path_buf();
						out.push(&new_name);
						out
					};
					if input == "R" {
						rename(new_path, path).context("failed to rename orginal")?;
						continue;
					} else {
						return Ok((Some(path), OnConflict::Ask));
					}
				}
				"p" | "P" => {
					print!("Enter prefix: ");
					let new_name = {
						let mut out = unwrap_or_print_err!(read_input(), continue);
						out.push_str(
							new_path
								.file_stem()
								.context("Failed to get file name")?
								.to_str()
								.context("Failed to convert  to str")?,
						);
						out
					};
					let path: PathBuf = {
						let mut out = new_path
							.clone()
							.parent()
							.context("Failed to get parent")?
							.to_path_buf();
						out.push(&new_name);
						out
					};
					if input == "P" {
						rename(new_path, path).context("failed to rename orginal")?;
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
