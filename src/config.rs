use std::path::PathBuf;

use anyhow::{Context, Result};
use figment::{
	providers::{Env, Format, Serialized, Toml},
	Figment,
};
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
	/// ReplaceSet,Editable
	pub sets: Vec<(ReplaceSet, bool)>,
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

	pub fn read() -> Result<Self> {
		let global_sets_dir = PathBuf::from("/etc/renameplus/sets.d");
		let user_sets_dir = {
			let mut out = dirs::config_dir().context("Failed to get config dir")?;
			out.push("renameplus");
			out.push("sets.d");
			out
		};
		if !user_sets_dir.exists() {
			if let Err(e) =
				std::fs::create_dir_all(&user_sets_dir).context("Failed to create user sets.d")
			{
				error!("{:?}", e)
			}
		}
		let global_config_path = "/etc/renameplus.toml";
		let user_config_path: PathBuf = {
			let mut out = dirs::config_dir().context("Failed to get config dir")?;
			out.push("renameplus");
			out.push("config.toml");
			out
		};
		let mut conf = Figment::new()
			.merge(Toml::file(global_config_path))
			.merge(Toml::file(user_config_path))
			.merge(Env::prefixed("renameplus_"));
		let sets: Vec<(ReplaceSet, bool)> = {
			let mut out: Vec<(ReplaceSet, bool)> = Self::builtin_sets()
				.into_iter()
				.map(|s| (s, false))
				.collect();
			if let Err(e) = find_sets(&mut out, global_sets_dir, false) {
				error!("{:?}", e)
			}
			if let Err(e) = find_sets(&mut out, user_sets_dir, true) {
				error!("{:?}", e);
			}
			out
		};
		conf = conf.merge(Serialized::default("sets", sets));
		conf.extract().context("Failed to parse config")
	}
}
fn find_sets(out: &mut Vec<(ReplaceSet, bool)>, dir: PathBuf, editable: bool) -> Result<()> {
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
					.with_context(|| format!("{}: Failed to parse toml", file.path().display()))?;
				out.push((set, editable))
			}
			Ok(())
		}
		Err(e) => {
			debug!("Skipping {}, because {}", dir.display(), e);
			Ok(())
		}
	}
}
