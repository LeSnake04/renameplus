use crate::RenamePlusGui;

impl RenamePlusGui {
	pub fn do_rename(&self) {
		self.data.rename().expect("Rename Failed");
	}
	pub fn validate(&self) -> String {
		let mut out: String = "".to_string();
		if self.files.is_empty() {
			out.push_str("Please add least one path.\n");
		}
		if !self.changes {
			out.push_str("No Changes Configured\n");
		}
		out
	}
}
