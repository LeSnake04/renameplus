use iced::{widget::Column, Alignment};

use crate::PresetDefault;

impl<Message> PresetDefault for Column<'_, Message> {
	fn preset_default(self) -> Self {
		self.align_items(Alignment::Center).spacing(20).padding(15)
	}
}
