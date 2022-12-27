use std::path::PathBuf;

use anyhow::Result;
use iced::{
	widget::{button, row as irow, text},
	Element,
};
use renameplus::rename::Rename;

#[derive(Default, Debug, Clone)]
pub struct FileList(pub Vec<FileItem>);

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct FileItem {
	pub path: PathBuf,
	pub new_path: PathBuf,
	deleted: bool,
}

#[derive(Debug, Clone)]
pub enum FileMessage {
	Delete,
	Replace(PathBuf),
}

impl FileItem {
	pub fn deleted(&self) -> bool {
		self.deleted
	}
	pub fn update_preview(&mut self, rename: &Rename) -> Result<()> {
		self.new_path = rename
			.get_new_path(&self.path)?
			.unwrap_or(self.path.clone());
		Ok(())
	}
	pub fn new(path: PathBuf, new_path: PathBuf) -> Self {
		Self {
			path,
			deleted: false,
			new_path,
		}
	}
	pub fn update(&mut self, msg: FileMessage) {
		match msg {
			FileMessage::Delete => self.deleted = true,
			FileMessage::Replace(f) => self.path = f,
		}
	}
	pub fn view(&self) -> Element<FileMessage> {
		irow![
			text(format!(
				"{} \n-> {}",
				self.path.display(),
				self.new_path.display()
			)),
			button(text("x")).on_press(FileMessage::Delete)
		]
		.into()
	}
}

impl IntoIterator for FileList {
	type Item = PathBuf;
	type IntoIter = std::vec::IntoIter<Self::Item>;
	fn into_iter(self) -> Self::IntoIter {
		// HACK is the collect really nessary?
		self.0
			.into_iter()
			.map(|f| f.path)
			.collect::<Vec<PathBuf>>()
			.into_iter()
	}
}
// impl FromIterator<(PathBuf, Option<PathBuf>)> for FileList {
// 	fn from_iter<T: IntoIterator<Item = (PathBuf, Option<PathBuf>)>>(iter: T) -> Self {
// 		Self(iter.into_iter().map(|p| FileItem::new(p.0, p.1)).collect())
// 	}
// }
