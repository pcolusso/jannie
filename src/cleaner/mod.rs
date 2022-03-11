use std::fs;
use std::fs::{DirEntry};
use std::io::Result;
use std::path::PathBuf;
use std::process::Command;
use guard::guard;

pub trait Cleaner {
  fn cleanable(&self, entry: &DirEntry) -> Option<u64>;
  fn clean(&self, entry: &DirEntry) -> Result<()>;
  fn name(&self) -> &'static str;
}

pub struct RustCleaner {}

impl Cleaner for RustCleaner {
  fn cleanable(&self, entry: &DirEntry) -> Option<u64> {
    simple_detect(entry, "Cargo.toml", "target")
  }

  fn clean(&self, entry: &DirEntry) -> Result<()> {
    Command::new("cargo")
      .args(["clean"])
      .current_dir(entry.path())
      .spawn()?;

    Ok(())
  }

  fn name(&self) -> &'static str {
    "Rust"
  }
}

pub struct NodeCleaner {}

impl Cleaner for NodeCleaner {
  fn name(&self) -> &'static str {
    "nodeJS"
  }

  fn cleanable(&self, entry: &DirEntry) -> Option<u64> {
    simple_detect(entry, "package-lock.json", "node_modules")
  }

  fn clean(&self, entry: &DirEntry) -> Result<()> {
    let delete_this_path_specifically = entry.path().join("node_modules");
    // NPM has no "clean" option, so we have to use remove directory specifically...
    fs::remove_dir_all(delete_this_path_specifically)?;
    Ok(())
  }
}

fn simple_detect(entry: &DirEntry, filename: &'static str, directory_name: &'static str) -> Option<u64> {
  guard!(let Ok(read_dir) = fs::read_dir(entry.path()) else { return None });

    for sub_entry in read_dir {
      guard!(let Ok(sub_entry) = sub_entry else { continue });
      guard!(let Ok(metadata) = fs::metadata(&sub_entry.path()) else { continue });

      if metadata.is_file() && sub_entry.file_name() == filename {
        match dir_size(entry.path().join(directory_name)) {
          Ok(size) => return Some(size),
          Err(_) => return None,
        }
      }
    }

    None
}

// Care of https://stackoverflow.com/questions/60041710/how-to-check-directory-size
fn dir_size(path: impl Into<PathBuf>) -> Result<u64> {
  fn dir_size(mut dir: fs::ReadDir) -> Result<u64> {
      dir.try_fold(0, |acc, file| {
          let file = file?;
          let size = match file.metadata()? {
              data if data.is_dir() => dir_size(fs::read_dir(file.path())?)?,
              data => data.len(),
          };
          Ok(acc + size)
      })
  }

  dir_size(fs::read_dir(path.into())?)
}