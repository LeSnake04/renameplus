use crate::RenamePlusGui;

impl RenamePlusGui {
	pub fn do_rename(&self) {
		self.data.rename().unwrap();
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

fn into_none_if<T>(cond: bool, obj: T) -> Option<T> {
	if cond {
		None
	} else {
		Some(obj)
	}
}
