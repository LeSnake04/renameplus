use anyhow::anyhow;
use error_log::ErrorLogAnyhow;
use iced::{
	widget::{button, checkbox, row, text, Row},
	Element, Length,
};
use renameplus::{ReplaceSetData, UsedReason};

use crate::PresetDefault;

#[derive(Debug)]
pub struct SetUi {
	pub set: ReplaceSetData,
	pub default: bool,
	pub edit: bool,
	pub name: String,
	pub search_draft: Vec<String>,
	pub replace_draft: String,
}

#[derive(Debug, Clone)]
pub enum SetUiMessage {
	Active(bool),
	ByDefault(bool),
	Edit,
	SaveDraft,
	DiscardDraft,
}

impl Default for SetUi {
	fn default() -> Self {
		Self {
			set: ReplaceSetData::default(),
			default: false,
			edit: false,
			name: "".to_string(),
			search_draft: vec![],
			replace_draft: "".to_string(),
		}
	}
}

impl SetUi {
	pub fn new(data: &ReplaceSetData, index: String) -> Self {
		Self {
			set: data.clone(),
			default: data.used == Some(UsedReason::Default),
			edit: false,
			name: index,
			search_draft: vec![],
			replace_draft: String::new(),
		}
	}
	pub fn update(
		&mut self,
		msg: SetUiMessage,
		err_log: &mut ErrorLogAnyhow<()>,
		update: &mut bool,
		set_default: &mut Option<bool>,
	) {
		let set = &mut self.set;
		match msg {
			SetUiMessage::Active(b) => {
				let used = b.then_some(UsedReason::Manual);
				set.used = used.clone();
				self.set.used = used;
				*update = true;
			}
			SetUiMessage::Edit => match set.editable {
				true => {
					self.edit = true;
					todo!("Edit set");
				}
				false => *err_log += anyhow!("{} is not editable!", set.set.name),
			},
			SetUiMessage::ByDefault(b) => {
				*set_default = Some(b);
			}
			SetUiMessage::SaveDraft => {
				self.set.set.search = self.search_draft.clone();
				self.set.set.replace = self.replace_draft.clone();
			}
			SetUiMessage::DiscardDraft => todo!(),
		}
	}
	pub fn view(&self) -> Element<SetUiMessage> {
		let edit: Row<SetUiMessage, _> = match (self.set.editable, self.edit) {
			(false, _) => row![],
			(true, false) => row![button("Edit").on_press(SetUiMessage::Edit)],
			(true, true) => row![button("Save").on_press(SetUiMessage::SaveDraft)],
		};
		let set = &self.set;
		row![
			checkbox(&set.set.name, set.used.is_some(), SetUiMessage::Active),
			text(&self.set.set.description).width(Length::Fill),
			edit,
			checkbox("Default", self.default, SetUiMessage::ByDefault)
		]
		.preset_default()
		.into()
	}
}
