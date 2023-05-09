use crate::RenamePlusGui;

impl RenamePlusGui {
	pub fn do_rename(&self) {
		let mut out = self.data.rename();
		out.display_fn_native_dialog();
		out.display_unwrap();
	}
	pub fn validate(&self) -> String {
		let mut out: String = "".to_string();
		if self.files.is_empty() {
			out.push_str("Please add least one path.\n");
		}
		if !self.changes {
			out.push_str("Nothing to Change\n");
		}
		out
	}
}
