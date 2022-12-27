use std::path::PathBuf;

use anyhow::Result;
use iced::{window::Event as WinEvent, Event};
use itertools::Itertools;
use log::error;
use native_dialog::{FileDialog, MessageDialog, MessageType};
use snake_helper::unwrap_or_print_err;

use crate::{
	file::{FileItem, FileMessage},
	replace::ReplaceMessage,
	RenamePlusGui,
};

#[derive(Debug, Clone)]
pub enum Message {
	AddPaths,
	Event(Event),
	FileMessage(usize, FileMessage),
	PrefixChanged(String),
	RemoveOutputDir,
	Run,
	SelectOutputDir,
	SuffixChanged(String),
	ToggleCopy(bool),
	ToggleDirs(bool),
	AddReplace,
	ReplaceMessage(usize, ReplaceMessage),
}
impl RenamePlusGui {
	pub(super) fn do_update(&mut self, message: Message) {
		let mut warns = "".to_string();
		let run = self.do_update_inner(message, &mut warns);
		if !warns.is_empty() {
			unwrap_or_print_err!(
				MessageDialog::new()
					.set_title("Warnig(s)")
					.set_type(MessageType::Warning)
					.show_alert(),
				return
			)
		}
		if let Err(e) = run {
			unwrap_or_print_err!(
				native_dialog::MessageDialog::new()
					.set_type(native_dialog::MessageType::Error)
					.set_text(&format!("{e:#?}"))
					.show_alert(),
				return
			);
		}
	}
	fn do_update_inner(&mut self, message: Message, warns: &mut String) -> Result<()> {
		let mut changed = false;
		let mut new_files: Vec<PathBuf> = vec![];
		match message {
			Message::Event(Event::Window(WinEvent::FileDropped(p))) => {
				new_files.push(p);
			}
			// Set hovered path
			Message::Event(Event::Window(WinEvent::FileHovered(p))) => self.hovered = Some(p),
			// Reset hovered path
			Message::Event(Event::Window(WinEvent::FilesHoveredLeft)) => self.hovered = None,
			Message::PrefixChanged(p) => {
				self.data.prefix = Some(p);
				changed = true
			}
			Message::SuffixChanged(p) => {
				self.data.suffix = Some(p);
				changed = true
			}
			Message::Run => self.do_rename(),
			Message::ToggleDirs(a) => self.data.dirs = a,
			Message::ToggleCopy(c) => self.data.copy = c,
			Message::AddPaths => self.folder_ask(&mut new_files),
			Message::SelectOutputDir => match FileDialog::new().show_open_single_dir() {
				Ok(Some(new)) => self.data.output_dir = Some(new),
				Ok(None) => warns.push_str("No dir selected"),
				Err(e) => error!("Failed to get path: {}", e),
			},
			Message::RemoveOutputDir => self.data.output_dir = None,
			Message::FileMessage(i, file_message) => {
				if let Some(file) = self.files.get_mut(i) {
					file.update(file_message);
				}
				// Remove files removed by user
				self.files.retain_mut(|file| !file.deleted());
			}
			// Ignore all others events
			Message::Event(_) => (),
			Message::AddReplace => self.new_replace(),
			Message::ReplaceMessage(i, msg) => {
				self.update_replace(i, msg);
				changed = true
			}
		}
		if !new_files.is_empty() {
			let mut new: Vec<FileItem> = vec![];
			for i in 0..new_files.len() {
				let file: PathBuf = new_files.remove(i);
				let new_path: PathBuf = unwrap_or_print_err!(self.data.get_new_path(&file), None)
					.unwrap_or(file.clone());
				new.push(FileItem::new(file, new_path));
			}
			self.files.append(&mut new);
			self.files = self.files.clone().into_iter().unique().collect()
		}
		if changed {
			self.any_changes();
			self.update_previews()?;
		};
		Ok(())
	}
	pub(super) fn update_replace(&mut self, i: usize, msg: ReplaceMessage) {
		match msg {
			ReplaceMessage::ChangeSearch(s) => {
				self.data.replace[i].0 = s.clone();
				self.replace_ui[i].search = s
			}
			ReplaceMessage::ChangeReplace(r) => {
				self.data.replace[i].1 = r.clone();
				self.replace_ui[i].replace = r
			}
			ReplaceMessage::Delete => {
				self.data.replace.remove(i);
				self.replace_ui.remove(i);
			}
		}
	}
}
