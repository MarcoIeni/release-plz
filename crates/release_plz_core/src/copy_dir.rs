use std::{fs, io, path::Path};

use anyhow::Context;
use ignore::{DirEntry, WalkBuilder};
use tracing::debug;

use crate::strip_prefix::strip_prefix;

fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> io::Result<()> {
    debug!(
        "creating symlink {:?} -> {:?}",
        &original.as_ref(),
        &link.as_ref()
    );

    #[cfg(unix)]
    return std::os::unix::fs::symlink(original, link);

    #[cfg(windows)]
    return std::os::windows::fs::symlink_file(original, link);
}

/// Copy directory preserving symlinks.
/// `to` is created if it doesn't exist.
pub fn copy_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> anyhow::Result<()> {
    let from = from.as_ref();
    anyhow::ensure!(from.is_dir(), "not a directory: {:?}", from);
    let dir_name = from
        .components()
        .last()
        .with_context(|| format!("invalid path {from:?}"))?
        .as_os_str();
    let to = to.as_ref().join(dir_name);
    debug!("copying directory from {:?} to {:?}", from, to);
    if !to.exists() {
        debug!("creating directory {:?}", to);
        fs::create_dir_all(&to).with_context(|| format!("cannot create directory {to:?}"))?;
    }

    copy_directory(from, to)?;

    Ok(())
}

/// `to` must exist.
#[tracing::instrument]
#[allow(clippy::filetype_is_file)] // we want to distinguish between files and symlinks
fn copy_directory(from: &Path, to: std::path::PathBuf) -> Result<(), anyhow::Error> {
    let walker = ignore::WalkBuilder::new(from)
        // Read hidden files
        .hidden(false)
        // Don't consider `.ignore` files.
        .ignore(false)
        .build();
    for entry in walker {
        let entry = entry.context("invalid entry")?;
        let destination = destination_path(&to, &entry, from)?;
        let file_type = entry.file_type().context("unknown file type")?;
        if file_type.is_dir() {
            if destination == to {
                continue;
            }
            debug!("creating directory {:?}", destination);
            fs::create_dir(&destination)
                .with_context(|| format!("cannot create directory {destination:?}"))?;
        } else if file_type.is_symlink() {
            let original_link = fs::read_link(entry.path())
                .with_context(|| format!("cannot read link {:?}", entry.path()))?;
            debug!("found symlink {:?} -> {:?}", entry.path(), original_link);
            let original_link = if original_link.is_relative() {
                original_link
            } else {
                let new_relative = strip_prefix(&original_link, from)?;
                to.join(new_relative)
            };
            create_symlink(&original_link, &destination).with_context(|| {
                format!(
                    "cannot create symlink {:?} -> {:?}",
                    &original_link, &destination
                )
            })?;
        } else if file_type.is_file() {
            debug!("copying file {:?} to {:?}", entry.path(), &destination);
            fs::copy(entry.path(), &destination).with_context(|| {
                format!("cannot copy file {:?} to {:?}", entry.path(), &destination)
            })?;
        }
    }
    Ok(())
}

fn destination_path(
    to: &Path,
    entry: &DirEntry,
    from: &Path,
) -> anyhow::Result<std::path::PathBuf> {
    let mut dest_path = to.to_path_buf();
    let relative = strip_prefix(entry.path(), from)?;
    dest_path.push(relative);
    Ok(dest_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_dir_copied_correctly() {
        let temp = tempfile::tempdir().unwrap();
        let subdir = "subdir";
        let subdir_path = temp.path().join(subdir);
        fs::create_dir(&subdir_path).unwrap();

        let file1 = subdir_path.join("file1");
        std::fs::write(&file1, "aaa").unwrap();
        let file2 = subdir_path.join("file2");
        create_symlink(&file1, file2).unwrap();

        let temp2 = tempfile::tempdir().unwrap();
        println!("from: {:?} to temp2: {:?}", subdir_path, temp2.path());
        copy_dir(subdir_path, temp2.path()).unwrap();
        let temp2_subdir = temp2.path().join(subdir);
        let new_file2 = temp2_subdir.join("file2");
        assert!(fs::symlink_metadata(&new_file2).unwrap().is_symlink());
        let link_target = fs::read_link(new_file2).unwrap();
        let file1_dest = temp2.path().join(subdir).join("file1");
        assert!(file1_dest.exists());
        assert_eq!(link_target, file1_dest);
    }

    #[test]
    fn is_symlink_created_if_file_exists() {
        let temp = tempfile::tempdir().unwrap();
        let file1 = temp.path().join("file1");
        let file2 = temp.path().join("file2");

        // file already exists
        std::fs::write(&file1, "aaa").unwrap();
        create_symlink(&file1, &file2).unwrap();
        let metadata = fs::symlink_metadata(&file2).unwrap();
        assert!(metadata.is_symlink());
        dbg!(metadata);
        let target = fs::read_link(file2).unwrap();
        assert_eq!(target, file1);
        assert_eq!(fs::read_to_string(target).unwrap(), "aaa");
        assert_eq!(fs::read_to_string(file1).unwrap(), "aaa");
    }

    #[test]
    fn is_symlink_created_before_file_exists() {
        let temp = tempfile::tempdir().unwrap();
        let file1 = temp.path().join("file1");
        let file2 = temp.path().join("file2");

        // file doesn't exist yet
        create_symlink(&file1, &file2).unwrap();
        std::fs::write(&file1, "aaa").unwrap();
        let metadata = fs::symlink_metadata(&file2).unwrap();
        assert!(metadata.is_symlink());
        dbg!(metadata);
        let target = fs::read_link(file2).unwrap();
        assert_eq!(target, file1);
        assert_eq!(fs::read_to_string(target).unwrap(), "aaa");
        assert_eq!(fs::read_to_string(file1).unwrap(), "aaa");
    }
}
