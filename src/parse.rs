use crate::{debug, info, use_err, warn, ArgMatches, PathBuf};
use std::fs::copy;
use std::fs::remove_file;
use std::fs::File;
use std::fs::Metadata;

use_err!();

pub fn run_with_path(
	file: &PathBuf,
	m: &ArgMatches,
	dry: bool,
	allow_dirs: bool,
	prefix: Option<&String>,
	suffix: Option<&String>,
	output_dir: Option<&PathBuf>,
) -> Result<()> {
	if !err!(file.try_exists(), "Error parsing file")? {
		Err(miette!(format!("File {} does not exist", file.display())))?;
	}
	let path: Result<PathBuf> =
		{
			or_err!(
				match file.file_name() {
					None =>
						match (file.parent(), allow_dirs,) {
							(None, false) => None,
							(Some(p), true) => {
								warn!("Skipped {} bevause it is a Directory. Use \'-r\' or \'--dirs\'", p.display());
								return Ok(());
							}
							(p, true) => p.map(|f| f.to_path_buf()),
							(p, false) => p.map(|r| r.to_owned()),
						},
					o => o.map(|f| f.into()),
				},
				"Failed to parse path {}",
				file.display()
			)
		};
	let get_parent = |file: PathBuf| -> Result<PathBuf> {
		match output_dir {
			None => or_err!(
				file.parent().map(|f| f.to_owned()),
				"Failed to get parent of {}",
				file.display()
			),
			o => or_err!(o.map(|d| d.to_owned()), "Failed to get output dir"),
		}
	};
	let mut new_name: String = path?.to_string_lossy().into();

	if let Some(p) = prefix {
		let mut out: String = p.to_owned();
		out.push_str(&new_name);
		new_name = out;
	}

	if let Some(s) = suffix {
		new_name.push_str(s);
	}

	let parent = get_parent(file.to_owned()).wrap_err("Failed to get parent of path")?;
	let new_path: PathBuf = {
		let mut out: PathBuf = parent.to_owned();
		out.push(&new_name);
		out
	};
	info!("{} -> {}", file.display(), new_path.display());
	if !dry {
		err!(
			copy(file, new_path),
			"Failed to copy file {}",
			file.display()
		)?;
	} else {
		debug!("Skipped because of \'--dry\'.");
	}

	if !dry && !err!(m.try_contains_id("copy"), "Failed to get field \'copy\'")? {
		err!(
			remove_file(file),
			"Failed to remove file {}",
			file.display()
		)?;
	} else {
		info!(
			"{} Skipped because of \'--dry\' or \'--copy \'.",
			file.display()
		);
		err!(parent.try_exists(), "Failed to verify path")?;
		let meta: Metadata = err!(
			err!(File::open(&parent), "get to to open file")?.metadata(),
			"Failed to get? metadata"
		)?;
		if !meta.is_dir() {
			Err(miette!(format!(
				"Parent {} not a directory",
				&parent.display()
			)))?
		}
		if meta.permissions().readonly() {
			Err(miette!(format!("Parent {} readonly", parent.display())))?;
		}
	}
	Ok(())
}
