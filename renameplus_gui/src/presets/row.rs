use super::PresetDefault;

impl<Message> PresetDefault for iced::widget::Row<'_, Message> {
	fn preset_default(self) -> Self {
		self.spacing(5)
	}
}
