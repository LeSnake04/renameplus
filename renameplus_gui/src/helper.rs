use std::path::PathBuf;

use anyhow::Result;
use log::error;
use native_dialog::{FileDialog, MessageDialog};

use crate::{RenamePlusGui, ReplaceItem, SetUi};

impl RenamePlusGui {
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
	pub(super) fn folder_ask(&mut self, new_files: &mut Vec<PathBuf>) {
		match FileDialog::new().show_open_multiple_file() {
			Ok(mut new) => {
				new.retain_mut(|path| {
					if path.is_dir() && !self.data.dirs {
						if let Ok(allow) = MessageDialog::new()
							.set_title("Folder Detected")
							.set_text("You selected a folder, enable renaming folders?")
							.show_confirm()
						{
							self.data.dirs = allow
						}
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
		// let last = self.data.replace.last().unwrap();
		self.replace_ui.push(ReplaceItem::new("", ""));
	}
	pub(super) fn reload_sets(&mut self) {
		self.sets.clear();
		for (name, set) in self.data.config.sets.iter() {
			self.sets
				.insert(name.clone(), SetUi::new(set, name.to_string()));
		}
	}
	pub(super) fn update_previews(&mut self) -> Result<()> {
		for file in self.files.iter_mut() {
			file.update_preview(&self.data)?
		}
		Ok(())
	}
}

// pub fn error_log_dialog(e: String, title: impl Into<String> + 'static) {
// 	let title = title.into();
// 	MessageDialog::new()
// 		.set_type(native_dialog::MessageType::Error)
// 		.set_title(&title)
// 		.set_text(&e)
// 		.show_alert()
// 		.expect("Failed to show error dialog");
// }
