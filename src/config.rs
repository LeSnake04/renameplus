use std::path::PathBuf;

use anyhow::{Context, Result};
use figment::{
	providers::{Env, Format, Toml},
	Figment,
};
use log::{debug, warn};
use serde::{Deserialize, Serialize};

use crate::{ErrorLog, ErrorLogAnyhow};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
	pub default_sets: Option<Vec<String>>,
	/// ReplaceSet, Used, Editable
	pub sets: Option<Vec<ReplaceSetData>>,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum UsedReason {
	Default,
	Manual,
	Dependant,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct ReplaceSetData {
	pub set: ReplaceSet,
	pub used: Option<UsedReason>,
	pub editable: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ReplaceSet {
	pub description: String,
	pub name: String,
	pub replace: String,
	pub search: Vec<String>,
}

impl Config {
	fn builtin_sets() -> [ReplaceSet; 1] {
		[ReplaceSet {
			name: String::from("no_whitespaces"),
			description: "replaces all whitespaces with underscores".to_string(),
			search: vec![" ".to_string()],
			replace: "_".to_string(),
		}]
	}

	fn find_sets(&self, out: &mut Vec<ReplaceSetData>, dir: PathBuf, editable: bool) -> Result<()> {
		match dir.read_dir() {
			Ok(g) => {
				for file in g {
					let file = file?;
					match file.path().extension() {
						Some(ext) if ext != "toml" => {
							warn!("{}: is not a toml", file.path().display());
							continue;
						}
						None => {
							warn!("{}: no extension found", file.path().display());
							continue;
						}
						_ => (),
					};
					let set: ReplaceSet = toml::from_str(&std::fs::read_to_string(file.path())?)
						.with_context(|| {
							format!("{}: Failed to parse toml", file.path().display())
						})?;
					out.push(ReplaceSetData {
						used: self.is_set_default(&set.name),
						set,
						editable,
					});
				}
				Ok(())
			}
			Err(e) => {
				debug!("Skipping {}, because {}", dir.display(), e);
				Ok(())
			}
		}
	}
	fn is_set_default(&self, name: &str) -> Option<UsedReason> {
		self.sets
			.as_ref()?
			.iter()
			.any(|s| s.set.name == name)
			.then_some(UsedReason::Default)
	}
	pub fn read() -> Result<ErrorLogAnyhow<Self>> {
		let mut err_log = ErrorLog::new();
		let global_sets_dir = PathBuf::from("/etc/renameplus/sets.d");
		let user_sets_dir = {
			let mut out = dirs::config_dir().context("Failed to get config dir")?;
			out.push("renameplus");
			out.push("sets.d");
			out
		};
		if !user_sets_dir.exists() {
			err_log +=
				std::fs::create_dir_all(&user_sets_dir).context("Failed to create user sets.d");
		}
		let global_config_path = "/etc/renameplus.toml";
		let user_config_path: PathBuf = {
			let mut out = dirs::config_dir().context("Failed to get config dir")?;
			out.push("renameplus");
			out.push("config.toml");
			out
		};
		let mut conf: Self = err_log
			.push_result(
				Figment::new()
					.merge(Toml::file(global_config_path))
					.merge(Toml::file(user_config_path))
					.merge(Env::prefixed("renameplus_"))
					.extract()
					.context("Failed to parse config"),
			)
			.unwrap_or_default();
		let sets: Vec<ReplaceSetData> = {
			let mut out: Vec<ReplaceSetData> = Self::builtin_sets()
				.into_iter()
				.map(|set| ReplaceSetData {
					used: conf.is_set_default(&set.name),
					set,
					editable: false,
				})
				.collect();
			err_log += conf.find_sets(&mut out, global_sets_dir, false);
			err_log += conf.find_sets(&mut out, user_sets_dir, true);
			out
		};
		conf.sets = Some(sets);
		err_log.set_ok(conf);
		Ok(err_log)
	}
}
