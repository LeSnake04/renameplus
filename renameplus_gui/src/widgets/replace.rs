use iced::{
	widget::{button, row, text, text_input},
	Element,
};

use crate::PresetDefault;

#[derive(Debug)]
pub struct ReplaceItem {
	pub replace: String,
	pub search: String,
}

#[derive(Debug, Clone)]
pub enum ReplaceMessage {
	ChangeSearch(String),
	// AddSearch(),
	ChangeReplace(String),
	Delete,
}

impl ReplaceItem {
	pub fn new(search: impl Into<String>, replace: impl Into<String>) -> Self {
		Self {
			search: search.into(),
			replace: replace.into(),
		}
	}
	pub fn view(&self) -> Element<'static, ReplaceMessage> {
		row![
			text_input("SEARCH", &self.search).on_input(ReplaceMessage::ChangeSearch),
			// tooltip(button(text("+")), "Add search", Position::FollowCursor),
			text_input("REPLACE", &self.replace).on_input(ReplaceMessage::ChangeReplace),
			button(text("X")).on_press(ReplaceMessage::Delete)
		]
		.preset_default()
		.into()
	}
}
