use std::path::PathBuf;

use anyhow::Result;
use log::error;
use native_dialog::{FileDialog, MessageDialog};
use snake_helper::unwrap_or_print_err;

use crate::{
	replace::{ReplaceItem, ReplaceMessage},
	RenamePlusGui,
};

impl RenamePlusGui {
	pub(super) fn folder_ask(&mut self, new_files: &mut Vec<PathBuf>) {
		let mut dir_asked = false;
		match FileDialog::new().show_open_multiple_file() {
			Ok(mut new) => {
				new.retain_mut(|path| {
					if path.is_dir() && !self.data.dirs {
						self.data.dirs = unwrap_or_print_err!(
							MessageDialog::new()
								.set_title("Folder Detected")
								.set_text("You selected a folder, enable renaming folders?")
								.show_confirm(),
							false
						);
						dir_asked = true;
						self.data.dirs
					} else {
						true
					}
				});
				new_files.append(&mut new);
			}
			Err(e) => error!("Failed to get path: {}", e),
		}
	}
	// fn new_replace(&mut self, search: impl Into<String>, replace: impl Into<String>) {
	pub(super) fn new_replace(&mut self) {
		self.data.push_replace("".to_string(), "".to_string());
		let last = self.data.replace.last().unwrap();
		self.replace_ui.push(ReplaceItem::new(&last.0, &last.1));
	}
	pub(super) fn any_changes(&mut self) {
		self.changes = self
			.data
			.prefix
			.as_ref()
			.map(|p| !p.is_empty())
			.unwrap_or(false)
			|| self
				.data
				.suffix
				.as_ref()
				.map(|s| !s.is_empty())
				.unwrap_or(false)
			|| self.data.replace.iter().any(|r| !r.0.is_empty());
	}
	pub(super) fn update_previews(&mut self) -> Result<()> {
		for file in self.files.iter_mut() {
			file.update_preview(&self.data)?
		}
		Ok(())
	}
}
