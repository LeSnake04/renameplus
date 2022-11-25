use format as f;
use log::{debug, error, info, warn};
use std::ffi::OsString;
use std::fs::{copy, rename};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, ensure, Context, Result};
use clap::ArgMatches;

use crate::args::OnConflict;

pub struct Rename {
    pub undo_on_err: bool,
    pub dry: bool,
    pub dirs: bool,
    pub files: Vec<PathBuf>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub copy: bool,
    pub on_conflict: OnConflict,
    pub fragile: bool,
    pub output_dir: Option<PathBuf>,
    pub output_files: Option<Vec<PathBuf>>,
}

#[derive(Debug, Clone)]
pub struct RenameOut {
    original: PathBuf,
    new_path: Option<PathBuf>,
}

impl TryFrom<ArgMatches> for Rename {
    type Error = anyhow::Error;

    fn try_from(m: clap::ArgMatches) -> Result<Self> {
        debug!("Parsing input args");
        let files: Vec<PathBuf> = m
            .try_get_many::<PathBuf>("file")?
            .context("Failed to get files")
            .map(move |v| v.cloned().collect())?;
        let output_files: Option<Vec<PathBuf>> = m
            .try_get_many::<PathBuf>("output-files")
            .context("failed to get argument \'output-files\'")?
            .map(move |v| v.cloned().collect());
        if let Some(ref o) = output_files {
            ensure!(o.is_empty(), "Need at least one output file");
            let output_file_last = o.last().context("output_files is empty")?;
            if o.len() < files.len() && !output_file_last.exists() {
                Err(anyhow!("Last entry of array does not exist"))?;
                ensure!(output_file_last.is_dir(), "Last entry isn't a directory");
            }
        }
        let undo_on_err: bool = *m
            .try_get_one("undo-on-err")
            .context("Failed to get argument \'undo-on-errr\'")?
            .ok_or(anyhow!("undo_on_err not set"))?;
        Ok(Self {
            undo_on_err,
            fragile: m
                .try_contains_id("fragile")
                .context("failed to get arg \'fragile\'")?
                || undo_on_err,
            dry: m
                .try_contains_id("dry")
                .context("Failed to get argument \'dry\'")?
                || undo_on_err,
            dirs: m
                .try_contains_id("dirs")
                .context("Failed to get argument \'dirs'")?,
            suffix: m
                .try_get_one("suffix")
                .context("Failed to get argument \'suffix\'")?
                .cloned(),
            prefix: m
                .try_get_one("prefix")
                .context("Failed to get argument \'prefix\'")?
                .cloned(),
            output_dir: m
                .try_get_one("output-dir")
                .context("Failed to get argument  \'output-dir\'")?
                .cloned(),
            files,
            copy: m
                .try_contains_id("copy")
                .context("Failed to get field \'copy\'")?,
            on_conflict: m
                .try_get_one::<OnConflict>("on-conflict")
                .context("Failed to get argument \'on-conflict\'")?
                .unwrap_or(&OnConflict::Skip)
                .to_owned(),
            output_files,
        })
    }
}

impl Rename {
    fn get_parent(&self, file: &Path) -> Result<PathBuf> {
        match &self.output_dir {
            // get parent if no output dir set.
            None => file
                .parent()
                .map(|f| f.to_owned())
                .context(format!("Failed to get parent of {}", file.display())),
            // return output dir if set.
            o => o.to_owned().context("Failed to get output dir"),
        }
    }
    pub fn parse(self) -> Result<()> {
        let mut history: Vec<RenameOut> = vec![];
        let mut out: Result<()> = Ok(());

        for file in &self.files {
            let curr_out = self.parse_path(file);
            match (curr_out, self.fragile, self.undo_on_err) {
                // Cancel if error occured and --fragile set.
                (Err(e), true, _) => {
                    out = Err(e);
                    break;
                }
                // Print error if --fragile not set.
                (Err(e), false, _) => error!("{:?}", e),
                // Push result to $history if moved and --und_on_err set.
                (Ok(r), _, true) => {
                    r.new_path.is_some().then(|| history.push(r));
                }
                _ => (),
            }
        }
        if out.is_err() && self.undo_on_err {
            for out in history {
                let new_path = match out.new_path.context("Path not set") {
                    // Print error if
                    Err(f) => {
                        error!("{f}");
                        continue;
                    }
                    //
                    Ok(p) => p,
                };
                // Perfom file Operations
                if let Err(e) = match self.copy {
                    true => std::fs::remove_file(new_path).context("Failed to remove file"),
                    false => {
                        std::fs::rename(new_path, out.original).context("Failed to rename file")
                    }
                } {
                    error!("{}", e)
                }
            }
        }
        Ok(())
    }
    fn parse_path(&self, file: &PathBuf) -> Result<RenameOut> {
        if !file.try_exists().context("Error parsing file")? {
            Err(anyhow!("File {} does not exist", file.display()))?;
        }
        let original: PathBuf = file.clone();
        let path: Result<PathBuf> = {
            match file.file_name() {
                None => match (file.parent(), self.dirs) {
                    (None, false) => None,
                    // Return early if $file is a folder and --dirs not set.
                    (Some(p), false) => {
                        warn!(
                            "Skipped {} bevause it is a Directory. Use \'-r\' or \'--dirs\'",
                            p.display()
                        );
                        return Ok(RenameOut {
                            original: file.clone(),
                            new_path: None,
                        });
                    }
                    (p, true) => p.map(|f| f.to_path_buf()),
                },
                o => o.map(|f| f.into()),
            }
            .context(format!("Failed to parse path {}", file.display()))
        };
        let mut new_name: OsString = {
            let out = path?;
            out.file_stem()
                .unwrap_or(out.file_name().context("Failed to get file name or stem")?)
                .to_owned()
        };

        if let Some(p) = &self.prefix {
            let mut out: OsString = p.into();
            out.push(&new_name);
            new_name = out;
        }

        if let Some(s) = &self.suffix {
            new_name.push(s);
        }

        let parent = self
            .get_parent(file)
            .context("Failed to get parent of path")?;
        let mut new_path: PathBuf = {
            let mut out: PathBuf = parent;
            out.push(&new_name);
            out
        };
        if new_path.exists() {
            let mut on_conflict: OnConflict = self.on_conflict.clone();
            loop {
                match on_conflict {
                    OnConflict::Ask => match self.conflict_ask(&new_path) {
                        Ok((Some(p), o)) => {
                            on_conflict = o;
                            new_path = p;
                            continue;
                        }
                        Ok((None, o)) => {
                            on_conflict = o;
                            continue;
                        }
                        Err(e) => {
                            Err(e)?;
                        }
                    },
                    OnConflict::Skip => {
                        return Ok(RenameOut {
                            original: file.to_owned(),
                            new_path: None,
                        })
                    }
                    _ => (),
                }
                break;
            }
        }
        info!("{} -> {}", file.display(), new_path.display());
        match (self.dry, self.copy) {
            (false, true) => {
                copy(file, &new_path).context(f!("Failed to copy file {}", file.display()))?;
            }
            (false, false) => {
                rename(file, &new_path).context(f!("Failed to rename file {}", file.display()))?
            }
            (true, _) => info!(
                "{} Skipped because of \'--dry\' or \'--copy \'.",
                file.display()
            ),
        }
        Ok(RenameOut {
            original,
            new_path: Some(new_path),
        })
    }
    pub fn verify_output_dir(&self) -> Result<&Self> {
        if let Some(o) = &self.output_dir {
            if !o.exists() {
                Err(anyhow!("Output dir {} doesn\'t exist", o.display()))?;
            }
        }
        Ok(self)
    }
}
