use std::{fs, io, path::Path};

use anyhow::Context;
use walkdir::WalkDir;

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
    dir_copy(subdir_path, temp2.path()).unwrap();
    let temp2_subdir = temp2.path().join(subdir);
    let new_file2 = temp2_subdir.join("file2");
    assert!(fs::symlink_metadata(&new_file2).unwrap().is_symlink());
    let link_target = fs::read_link(new_file2).unwrap();
    let file1_dest = temp2.path().join(subdir).join("file1");
    assert!(file1_dest.exists());
    assert_eq!(link_target, file1_dest);
}

#[test]
fn is_symlink_created() {
    let temp = tempfile::tempdir().unwrap();
    let file1 = temp.path().join("file1");
    let file2 = temp.path().join("file2");
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

fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> io::Result<()> {
    #[cfg(unix)]
    return std::os::unix::fs::symlink(original, link);

    #[cfg(windows)]
    return std::os::windows::fs::symlink_file(original, link);
}

/// Copy directory preserving symlinks.
/// `to` is created if it doesn't exist.
pub fn dir_copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> anyhow::Result<()> {
    let from = from.as_ref();
    let dir_name = from
        .components()
        .last()
        .context("invalid path")?
        .as_os_str();
    anyhow::ensure!(from.is_dir(), "not a directory: {:?}", from);
    let to = to.as_ref().join(dir_name);
    println!("from: {:?}", from);
    println!("to: {:?}", to);
    if !to.exists() {
        fs::create_dir_all(&to).context("cannot create directory")?;
    }
    for entry in WalkDir::new(from) {
        println!("entry: {:?}", entry);
        let entry = entry.context("invalid entry")?;
        let file_type = entry.file_type();
        let mut dest_path = to.clone();
        let relative = entry.path().strip_prefix(from).unwrap();
        dest_path.push(relative);

        if file_type.is_dir() {
            if dest_path == to {
                continue;
            }
            fs::create_dir(&dest_path)
                .with_context(|| format!("cannot create directory {dest_path:?}"))?;
        } else if file_type.is_symlink() {
            // TODO: can we create symlink if new path exists?
            let original_link = fs::read_link(entry.path())
                .with_context(|| format!("cannot read link {:?}", entry.path()))?;
            let new_relative = original_link.strip_prefix(from)?;
            let new_link = to.join(new_relative);
            create_symlink(new_link, &dest_path)
                .with_context(|| format!("cannot create symlink {:?}", dest_path))?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &dest_path)
                .with_context(|| format!("cannot copy file {:?}", entry.path()))?;
        }
    }

    Ok(())
}
