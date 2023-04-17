use std::path::PathBuf;

pub use crate::update::Message;
use crate::{helper::error_log_dialog, FileItem, PresetDefault, ReplaceItem, SetUi};
use iced::{
	executor,
	widget::{
		button, column as icolumn, row as irow, scrollable, text, text_input, toggler, tooltip,
		Column,
	},
	Alignment, Application, Color, Command, Theme,
};

use renameplus::{rename::Rename, Config, ErrorLogAnyhow, PrintMode};

#[derive(Debug, Default)]
pub struct RenamePlusGui {
	pub changes: bool,
	pub data: Rename,
	pub files: Vec<FileItem>,
	pub hovered: Option<PathBuf>,
	pub replace_ui: Vec<ReplaceItem>,
	pub sets: Vec<SetUi>,
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
				let config = err_log.push_result(Config::read());
				if let Some(mut c) = config {
					err_log
						.prepend_errors(&mut c)
						.join_on_display("\n\n")
						.print_fn(|e| error_log_dialog(e, "Failed to parse config"))
						.display_mode(PrintMode::Debug);
					*err_log.ok_mut() = c.take_ok();
				}
				err_log.unwrap_or_display().expect("Failed to get config")
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
		scrollable(
			if !self.sets_overlay {
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
				// let modal_state = modal::State::new(());
				// overlay::;

				// Modal::new(true, Text::new("underlay"), || {
				// icolumn![text("overlay")].into()
				// })

				icolumn![
					button("Add Files").on_press(Message::AddPaths),
					irow![
						text(drop_hint),
						text(match self.data.output_dir {
							Some(ref o) => o.display().to_string(),
							None => "<Using File source path>".to_string(),
						}),
						button(text("Set output dir")).on_press(Message::SelectOutputDir),
						button(text("X")).on_press(Message::RemoveOutputDir)
					]
					.preset_default(),
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
					replace,
					button(text("Add replace filter")).on_press(Message::AddReplace),
					button(text("Select Sets")).on_press(Message::ShowSetsSelect),
					run_button,
					validate_msgs,
					#[cfg(debug_assertions)]
					text(format!("{self:#?}")),
					// #[cfg(all(not(windows), not(unix)))]
					// text("Replace not supported on your OS"),
					// #[cfg(all(windows, unix))]
				]
			} else {
				let sets: Column<Message> = {
					let mut out = Column::new();
					for (i, set) in self.sets.iter().enumerate() {
						out = out.push(set.view().map(move |msg| Message::SetMessage(i, msg)));
					}
					out
				};
				// Show overlay
				icolumn![
					sets,
					irow![
						button(text("Cancel")).on_press(Message::HideSetsSelect),
						button("Save")
					]
					.preset_default(),
					#[cfg(debug_assertions)]
					text(format!("{self:#?}")),
				]
			}
			.align_items(Alignment::Center)
			.spacing(20)
			.padding(15),
		)
		.into()
	}
}
