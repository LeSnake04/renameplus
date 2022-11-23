mod errors;

use std::{
	fmt::Display,
	path::PathBuf,
	time::{Duration, Instant},
};

use eframe::{
	egui::{CentralPanel, DroppedFile, TopBottomPanel},
	epaint::Color32,
	run_native, App, NativeOptions,
};
use native_dialog::FileDialog;

use crate::errors::gui::{GuiError as Error, GuiResult as Result};

pub fn main() {
	let options: NativeOptions = NativeOptions::default();
	run_native(
		"RenamePlus",
		options,
    Box::new(|_| {Box::new(MainUi::default()))
	);

#[derive(Default)]
struct MainUi {
	files: Vec<PathBuf>,
	dropped_files: Vec<DroppedFile>,
	notifications: Vec<Notification>,
}

struct Notification {
	text: String,
	level: NotificationLevel,
	start: Instant,
	duration: Duration,
}

impl Notification {
	fn new(
		text: impl Into<String>,
		duration: Duration,
		level: impl Into<NotificationLevel>,
	) -> Self {
		let text = text.into();
		let level = level.into();
		Self {
			text,
			start: Instant::now(),
			duration,
			level,
		}
	}
}

#[derive(Debug, Clone, Copy)]
enum NotificationLevel {
	Info = 0,
	Warn = 1,
	Error = 2,
}

impl From<NotificationLevel> for Color32 {
	fn from(input: NotificationLevel) -> Self {
		match input {
			NotificationLevel::Info => Color32::LIGHT_BLUE,
			NotificationLevel::Warn => Color32::GOLD,
			NotificationLevel::Error => Color32::RED,
		}
	}
}

impl Display for NotificationLevel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			NotificationLevel::Info => "Info",
			NotificationLevel::Warn => "Warning",
			NotificationLevel::Error => "Error",
		})
	}
}

impl App for MainUi {
	fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
		TopBottomPanel::top("notification_panel").show(ctx, |ui| {
			// let mut table = TableBuilder::new(ui);
			for mut i in 0..self.notifications.len() {
				let n: &Notification = self.notifications.get(i).unwrap();
				// table.body(|mut body| {
				// 	body.rows(18.0, self.notifications.len(), |i, mut row| {
				// 		row.col(|ui: &mut eframe::egui::Ui| {
				// 			ui.colored();
				// 		});
				// 	})
				// });
				if n.duration.as_millis() > n.start.elapsed().as_millis() {
					ui.colored_label(n.level, format!("{}: {}", &n.level.to_string(), &n.text));
				} else {
					self.notifications.remove(i);
					i -= 1;
				}
			}
		});
		CentralPanel::default().show(ctx, |ui| {
			ui.label("drag and drop files into window to rename");
			if ui.button("Open File").clicked() {
				let mut files = FileDialog::new().show_open_multiple_file().unwrap();
				if !files.is_empty() {
					self.files.append(&mut files);
				}
			}
			for i in 0..self.dropped_files.len() {
				let f = &self.dropped_files.get(i).unwrap().clone();
				match &f.path {
					Some(p) => {
						self.notify(
							format!("Added Path {}", p.display()),
							Duration::from_secs(5),
							"i",
						)
						.unwrap();
						self.files.push(p.to_owned());
						self.dropped_files.swap_remove(i);
					}
					None => {
						self.notify(
							format!("Failed to find path to file {}", f.name),
							Duration::from_secs(5),
							"w",
						)
						.unwrap();
						continue;
					}
				}
			}
		});
		// Collect dropped files:
		if !ctx.input().raw.dropped_files.is_empty() {
			self.dropped_files = ctx.input().raw.dropped_files.clone();
		}
	}
}

impl MainUi {
	fn notify(
		&mut self,
		msg: impl Into<String>,
		duration: Duration,
		level: impl AsRef<str>,
	) -> Result<()> {
		let level = match level.as_ref() {
			"info" | "i" => NotificationLevel::Info,
			"warning" | "warn" | "w" => NotificationLevel::Warn,
			"error" | "e" => NotificationLevel::Error,
			s => return Err(Error::InvalidNotificationLevel(s.to_string())),
		};
		self.notifications
			.push(Notification::new(msg, duration, level));
		Ok(())
	}
}
