use std::path::PathBuf;

pub use crate::update::Message;
use crate::{file::FileItem, replace::ReplaceItem};
use iced::{
	executor,
	widget::{
		button, column as icolumn, row as irow, scrollable, text, text_input, toggler, tooltip,
		Column,
	},
	Alignment, Application, Color, Command, Theme,
};
use iced_native::Overlay;
use renameplus::rename::Rename;

#[derive(Debug, Default)]
pub struct RenamePlusGui {
	pub data: Rename,
	pub files: Vec<FileItem>,
	pub hovered: Option<PathBuf>,
	pub replace_ui: Vec<ReplaceItem>,
	pub changes: bool,
}

impl Application for RenamePlusGui {
	type Executor = executor::Default;
	type Message = Message;
	type Theme = Theme;
	type Flags = ();

	fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
		let mut out = Self::default();
		out.new_replace();
		(out, Command::none())
	}

	fn subscription(&self) -> iced::Subscription<Self::Message> {
		iced::subscription::events().map(Message::Event)
	}

	fn theme(&self) -> Theme {
		Theme::Dark
	}

	fn title(&self) -> String {
		"RenamePlus".to_string()
	}

	fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
		self.do_update(message);
		Command::none()
	}

	fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
		let (validate_msgs, run_button) = {
			let mut out = (
				text("Ready".to_string()).style(Color::from_rgb8(20, 200, 0)),
				button(text("Run")),
			);
			match self.validate() {
				v if !v.is_empty() => out.0 = text(v).style(Color::from_rgb8(170, 80, 0)),
				_ => out.1 = out.1.on_press(Message::Run),
			}
			out
		};
		let files: Column<Message> = {
			let mut out = Column::new();
			for (i, file) in self.files.iter().enumerate() {
				out = out.push(file.view().map(move |msg| Message::FileMessage(i, msg)));
			}
			out
		};
		let replace: Column<Message> = {
			let mut out = Column::new();
			for (i, rename) in self.replace_ui.iter().enumerate() {
				out = out.push(
					rename
						.view()
						.map(move |msg| Message::ReplaceMessage(i, msg)),
				);
			}
			out
		};
		let drop_hint: String = match self.hovered {
			Some(ref f) => format!("Drop file to add path(s): {}", f.display()),
			None => String::from("\n"),
		};
		// let overlay = Overlay::new();
		scrollable(
			icolumn![
				button("Add Files").on_press(Message::AddPaths),
				irow![
					text(drop_hint),
					text(match self.data.output_dir {
						Some(ref o) => o.display().to_string(),
						None => "<Using File source path>".to_string(),
					}),
					button(text("Set output dir")).on_press(Message::SelectOutputDir),
					button(text("-")).on_press(Message::RemoveOutputDir)
				],
				tooltip(
					text_input(
						"PREFIX",
						match self.data.prefix {
							Some(ref s) => s,
							None => "",
						},
						Message::PrefixChanged
					),
					"Text to add before file names",
					tooltip::Position::Right,
				),
				tooltip(
					text_input(
						"SUFFIX",
						match self.data.suffix {
							Some(ref s) => s,
							None => "",
						},
						Message::SuffixChanged
					),
					"Text to add after file names",
					tooltip::Position::Right,
				),
				toggler(
					"Allow directories".to_string(),
					self.data.dirs,
					Message::ToggleDirs,
				),
				toggler(
					"Copy instead of renaming".to_string(),
					self.data.copy,
					Message::ToggleCopy
				),
				files,
				button(text("Add replace filter")).on_press(Message::AddReplace),
				run_button,
				validate_msgs,
				replace,
				#[cfg(debug_assertions)]
				text(format!("{self:#?}")),
				// #[cfg(all(not(windows), not(unix)))]
				// text("Replace not supported on your OS"),
				// #[cfg(all(windows, unix))]
			]
			.align_items(Alignment::Center)
			.spacing(20),
		)
		.into()
	}
}
