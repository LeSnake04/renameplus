use crate::{irow, PresetDefault};
use iced::{
	widget::{button, checkbox, text, Row},
	Element, Length,
};
use renameplus::{ReplaceSetData, UsedReason};

#[derive(Debug)]
pub struct SetUi {
	pub set: ReplaceSetData,
	pub default: bool,
	pub edit: bool,
	pub index: usize,
}

#[derive(Debug, Clone)]
pub enum SetUiMessage {
	Toggle(bool),
	ByDefault(bool),
	Edit,
}

impl SetUi {
	pub fn new(data: &ReplaceSetData, index: usize) -> Self {
		Self {
			set: data.clone(),
			default: data.used == Some(UsedReason::Default),
			edit: false,
			index,
		}
	}
	pub fn view(&self) -> Element<SetUiMessage> {
		let edit: Row<SetUiMessage, _> = match self.set.editable {
			false => irow![],
			true => irow![button("Edit").on_press(SetUiMessage::Edit)],
		};
		let set = &self.set;
		irow![
			checkbox(&set.set.name, set.used.is_some(), SetUiMessage::Toggle),
			text(&self.set.set.description).width(Length::Fill),
			edit,
			checkbox("Default", self.default, SetUiMessage::ByDefault)
		]
		.preset_default()
		.into()
	}
}
