use std::path::PathBuf;

pub use crate::update::Message;
use crate::{FileItem, ReplaceItem, SetUi};
use iced::{executor, widget::scrollable, Application, Command, Theme};

use error_log::{ErrorLogAnyhow, FormatMode};
use iced_aw::Modal;
use renameplus::{rename::Rename, Config};

#[derive(Debug, Default)]
pub struct RenamePlusGui {
	pub changes: bool,
	pub data: Rename,
	pub files: Vec<FileItem>,
	pub hovered: Option<PathBuf>,
	pub replace_ui: Vec<ReplaceItem>,
	pub sets: Vec<SetUi>,
	pub new_set: SetUi,
	pub sets_overlay: bool,
}

impl Application for RenamePlusGui {
	type Executor = executor::Default;
	type Message = Message;
	type Theme = Theme;
	type Flags = ();

	fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
		let mut out = {
			let mut err_log = ErrorLogAnyhow::<Config>::new();
			let config = {
				err_log
					.append_entries(&mut Config::read())
					.display_mode(FormatMode::Debug)
					.delimiter("\n\n");
				err_log.display_fn_native_dialog();
				err_log.display_unwrap_or_default()
			};
			Self {
				data: Rename {
					config,
					..Default::default()
				},
				..Default::default()
			}
		};
		out.new_replace();
		out.reload_sets();
		(out, Command::none())
	}
	fn title(&self) -> String {
		"RenamePlus".to_string()
	}

	fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
		self.do_update(message);
		Command::none()
	}

	fn view(&self) -> iced::Element<'_, self::Message, iced::Renderer<Self::Theme>> {
		scrollable(Modal::new(self.sets_overlay, self.view_underlay(), || {
			self.view_overlay()
		}))
		.into()
	}

	fn theme(&self) -> Theme {
		Theme::Dark
	}

	fn subscription(&self) -> iced::Subscription<Self::Message> {
		iced::subscription::events().map(Message::Event)
	}
}
