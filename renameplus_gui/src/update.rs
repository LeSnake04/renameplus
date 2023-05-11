use std::path::PathBuf;

use anyhow::{anyhow, Result};
use error_log::ErrorLogAnyhow;
use iced::{
	keyboard::{Event as KeyEvent, KeyCode},
	window::Event as WinEvent,
	Event,
};
use itertools::Itertools;
use native_dialog::{FileDialog, MessageDialog, MessageType};
use snake_helper::{unwrap_or_print_err, unwrap_some_or};

use crate::{FileItem, FileMessage, RenamePlusGui, ReplaceMessage, SetUiMessage};

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
	ShowSetsSelect,
	HideSetsSelect,
	SetMessage(String, SetUiMessage),
	NewSetMessage(SetUiMessage),
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
				{}
			);
		}
	}
	fn do_update_inner(
		&mut self,
		message: Message,
		warns: &mut String,
	) -> Result<ErrorLogAnyhow<()>> {
		let mut err_log = ErrorLogAnyhow::new();
		let mut changed = false;
		let mut new_files: Vec<PathBuf> = vec![];
		match message {
			Message::Event(Event::Keyboard(KeyEvent::KeyPressed {
				key_code: KeyCode::Escape,
				..
			})) => self.sets_overlay = false,
			Message::SetMessage(name, msg) => self.update_set(name, msg, &mut err_log),
			Message::NewSetMessage(msg) => {
				let mut err_log = ErrorLogAnyhow::new();
				let mut update = false;
				let mut set_default = None;
				self.new_set
					.update(msg, &mut err_log, &mut update, &mut set_default);
				err_log.display_ok();
			}
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
				Err(e) => err_log += anyhow!("Failed to get path: {}", e),
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
			Message::ShowSetsSelect => self.sets_overlay = true,
			Message::HideSetsSelect => self.sets_overlay = false,
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
		Ok(err_log)
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
	pub(super) fn update_set(
		&mut self,
		name: String,
		msg: SetUiMessage,
		err_log: &mut ErrorLogAnyhow<()>,
	) {
		// let set_name = self.sets.get(&name).name.clone();
		let set = self.data.config.sets.get(&name).unwrap();
		let mut update = false;
		let mut set_default = None;
		self.sets
			.get_mut(&name)
			.unwrap()
			.update(msg, err_log, &mut update, &mut set_default);
		if let Some(state) = set_default {
			let name = set.set.name.clone();
			self.make_set_default(&name, state);
			self.sets.get_mut(&name).unwrap().default = true;
		}
		if update {
			self.reload_sets();
		}
	}
	fn make_set_default(&mut self, set: impl AsRef<str>, state: bool) {
		let set = set.as_ref();
		let default_sets = unwrap_some_or!(self.data.config.default_sets.as_mut(), return);
		let default_set_i = default_sets
			.iter()
			.position(|s| s == set)
			.expect("Not found in default set");

		match state {
			true => default_sets.push(set.to_string()),
			false => {
				default_sets.remove(default_set_i);
			}
		}
	}
}
