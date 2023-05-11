use iced::{
	widget::{button, row, text, text_input, toggler, tooltip, Column, Container},
	Color, Element,
};
use iced_aw::Card;

use crate::{col, Message, PresetDefault, RenamePlusGui};

impl RenamePlusGui {
	pub fn view_overlay(&self) -> Element<'_, Message> {
		let sets: Column<Message> = {
			let mut out = Column::new();
			for (i, set) in self.sets.iter().enumerate() {
				out = out.push(set.view().map(move |msg| Message::SetMessage(i, msg)));
			}
			out
		};
		Card::new(
			text("Sets"),
			col![
				sets,
				row![self.new_set.view().map(Message::NewSetMessage), button(text("+"))],
				row![
					button(text("Cancel")).on_press(Message::HideSetsSelect),
					button("Save")
				]
				.preset_default(),
				#[cfg(debug_assertions)]
				text(format!("{self:#?}")),
			]
			.preset_default(),
		)
		.into()
	}
	pub fn view_underlay(&self) -> Container<'_, Message> {
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

		Container::new(
			col![
				button("Add Files").on_press(Message::AddPaths),
				row![
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
					)
					.on_input(Message::PrefixChanged),
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
					)
					.on_input(Message::SuffixChanged),
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
			.preset_default(),
		)
	}
}
