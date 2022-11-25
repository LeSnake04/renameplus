use std::path::PathBuf;

use anyhow::{Context, Result};
use flexi_logger::{LogSpecification, Logger};
use iced::{
	executor,
	widget::{button, text, text_input, tooltip, Button, Column, Text},
	window::Event as WinEvent,
	Alignment, Application, Command, Event, Settings, Theme,
};
use itertools::Itertools;

mod errors;

#[derive(Debug, Default)]
struct RenamePlusGui {
	prefix: String,
	suffix: String,
	hovered: Option<PathBuf>,
	files: Vec<PathBuf>,
}

// impl Default for RenamePlusGui {
// 	fn default() -> Self {
// 		Self {
// 			prefix: "prefix...".into()
// 			suffix: ""
// 			files: Vec::new() }
// 	}
// }

fn main() -> Result<()> {
	Logger::with(
		LogSpecification::env_or_parse("renameplus_gui=debug, off")
			.context("Failed to parse logger config")?,
	)
	.start()
	.context("Failed to init logger")?;
	RenamePlusGui::run(Settings::with_flags(())).context("Faild to start gui")?;
	Ok(())
}

#[derive(Debug, Clone)]
enum Message {
	Event(Event),
	PrefixChanged(String),
	SuffixChanged(String),
}

impl Application for RenamePlusGui {
	type Executor = executor::Default;
	type Message = Message;
	type Theme = Theme;
	type Flags = ();

	fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
		(Self::default(), Command::none())
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
		match message {
			Message::Event(Event::Window(WinEvent::FileDropped(p))) => {
				self.files.push(p);
				self.files = self.files.clone().into_iter().unique().collect();
			}
			Message::Event(Event::Window(WinEvent::FileHovered(p))) => self.hovered = Some(p),
			// Reset hovered path
			Message::Event(Event::Window(WinEvent::FilesHoveredLeft)) => self.hovered = None,
			Message::PrefixChanged(p) => self.prefix = p,
			Message::SuffixChanged(p) => self.suffix = p,
			// Ignore all others events
			Message::Event(_) => (),
		}
		Command::none()
	}

	fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
		Column::new()
			.align_items(Alignment::Center)
			.push(tooltip(
				text_input("PREFIX", &self.prefix, Message::PrefixChanged),
				"File Prefix",
				tooltip::Position::FollowCursor,
			))
			.push(tooltip(
				text_input("SUFFIX", &self.suffix, Message::SuffixChanged),
				"File suffix",
				tooltip::Position::FollowCursor,
			))
			.push(Text::new(format!("Hovered: {:#?}", self.hovered)))
			.push(Text::new(format!("Droped: {:#?}", self.files)))
			.push(button(text("Run")))
			.into()
	}
}

// impl run
