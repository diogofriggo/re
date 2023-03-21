use clap::Parser;
use color_eyre::eyre::eyre;
use color_eyre::eyre::ContextCompat;
use color_eyre::eyre::Report;
use color_eyre::Result;
use regex::Regex;
use std::fs;
use std::fs::DirEntry;
use std::path::Path;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    from: String,
    to: String,

    #[arg(short, default_value_t = false)]
    force: bool,

    #[arg(short, default_value_t = false)]
    verbose: bool,
}

#[derive(Debug)]
struct Utf8Path {
    file_name: String,
    parent: PathBuf,
}

impl TryFrom<DirEntry> for Utf8Path {
    type Error = Report;

    fn try_from(entry: DirEntry) -> Result<Self> {
        match entry.metadata()?.is_dir() {
            true => Err(eyre!("skipping directory")),
            false => {
                let path = entry.path();
                let file_name = path
                    .file_name()
                    .context(format!("expected {path:?}  to have a name"))?
                    .to_str()
                    .context(format!("{path:?} is not an UTF-8 string"))?;
                Ok(Self {
                    file_name: file_name.into(),
                    parent: path
                        .parent()
                        .context(format!("expected {path:?} to have a parent"))?
                        .into(),
                })
            }
        }
    }
}

fn rename(folder: &Path, old_file_name: &str, new_file_name: &str) -> Result<()> {
    let mut old_path = folder.to_path_buf();
    old_path.push(old_file_name);

    let mut new_path = folder.to_path_buf();
    new_path.push(new_file_name);

    std::fs::rename(old_file_name, new_file_name)?;

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let folder = std::env::current_dir()?;

    let from = Regex::new(&args.from)?;
    let _to_can_be_converted_to_a_regex = Regex::new(&args.to)?;
    if !args.force {
        println!("Changes to be applied if you pass -f:");
    }
    // flatten converts vec![Some(_), None, Some(_)] into vec![_, _]
    for entry in fs::read_dir(&folder)? {
        if let Err(err) = &entry {
            if args.verbose {
                println!("while reading entry from {folder:?}: {err}")
            }
            continue;
        }

        let entry = entry?;
        match Utf8Path::try_from(entry) {
            Ok(path) => {
                let old_file_name = &path.file_name;
                let new_file_name = from.replace(&path.file_name, &args.to);
                if args.force {
                    if let Err(err) = rename(&path.parent, old_file_name, &new_file_name) {
                        if args.verbose {
                            println!("Could not rename {path:?} due to {err}");
                        }
                    }
                // we don't want to report identity renames
                } else if *old_file_name != *new_file_name {
                    println!("{old_file_name} -> {new_file_name}");
                }
            }
            Err(err) => {
                if args.verbose {
                    println!("while parsing Utf8Path: {err}")
                }
            }
        }
    }
    Ok(())
}
