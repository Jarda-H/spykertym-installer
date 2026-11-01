use super::*;
use std::fs;
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("spyker-test-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("game")).unwrap();
    dir
}

fn item(dir: &Path, name: &str, content: &str) -> InstallItem {
    let from = dir.join("src").join(name);
    fs::write(&from, content).unwrap();
    InstallItem {
        from: from.to_str().unwrap().into(),
        to: dir.join("game").join(name).to_str().unwrap().into(),
        own: false,
    }
}

fn install(items: &[InstallItem]) -> Result<Vec<String>, InstallError> {
    stage_files(items)?;
    commit_files(items)
}

fn game_files(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir.join("game"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_str().unwrap().to_string())
        .collect();
    names.sort();
    names
}

fn read(dir: &Path, name: &str) -> String {
    fs::read_to_string(dir.join("game").join(name)).unwrap()
}

#[test]
fn md5_matches_across_read_chunks() {
    let dir = temp_dir("md5");
    let small = dir.join("small");
    fs::write(&small, "abc").unwrap();
    assert_eq!(
        md5_of_file(small.to_str().unwrap()).unwrap(),
        "900150983cd24fb0d6963f7d28e17f72"
    );

    let content = vec![7u8; 3 * 1024 * 1024 + 123];
    let big = dir.join("big");
    fs::write(&big, &content).unwrap();
    assert_eq!(
        md5_of_file(big.to_str().unwrap()).unwrap(),
        format!("{:x}", md5::compute(&content))
    );
    assert!(md5_of_file("/nonexistent/file").is_err());
}

#[test]
fn locked_file_is_retried() {
    let mut calls = 0;
    let result = retry_if_locked(|| {
        calls += 1;
        if calls < 3 {
            Err(std::io::Error::from_raw_os_error(32))
        } else {
            Ok("done")
        }
    });
    assert_eq!(result.unwrap(), "done");
    assert_eq!(calls, 3);
}

#[test]
fn permanently_locked_file_gives_up() {
    let mut calls = 0;
    let result: std::io::Result<()> = retry_if_locked(|| {
        calls += 1;
        Err(std::io::Error::from_raw_os_error(32))
    });
    assert_eq!(result.unwrap_err().raw_os_error(), Some(32));
    assert_eq!(calls, 6);
}

#[test]
fn other_errors_are_not_retried() {
    let mut calls = 0;
    let result: std::io::Result<()> = retry_if_locked(|| {
        calls += 1;
        Err(std::io::Error::from_raw_os_error(5))
    });
    assert!(result.is_err());
    assert_eq!(calls, 1);
}

#[test]
fn update_keeps_the_original_backup() {
    let dir = temp_dir("update");
    fs::write(dir.join("game/a.pak"), "ORIGINAL").unwrap();

    install(&[item(&dir, "a.pak", "CZ-v1")]).unwrap();
    assert_eq!(read(&dir, "a.pak"), "CZ-v1");
    assert_eq!(read(&dir, "a.pak.backup"), "ORIGINAL");

    install(&[item(&dir, "a.pak", "CZ-v2")]).unwrap();
    assert_eq!(read(&dir, "a.pak"), "CZ-v2");
    assert_eq!(read(&dir, "a.pak.backup"), "ORIGINAL");
    assert_eq!(game_files(&dir), ["a.pak", "a.pak.backup"]);
}

#[test]
fn new_file_gets_no_backup() {
    let dir = temp_dir("new");
    install(&[item(&dir, "cz.pak", "CZ")]).unwrap();
    assert_eq!(game_files(&dir), ["cz.pak"]);
}

#[cfg(unix)]
#[test]
fn failed_swap_rolls_back_every_file() {
    use std::os::unix::fs::PermissionsExt;

    let dir = temp_dir("rollback");
    let locked_dir = dir.join("game/locked");
    fs::create_dir_all(&locked_dir).unwrap();
    fs::write(dir.join("game/a.pak"), "ORIGINAL-A").unwrap();
    fs::write(locked_dir.join("b.pak"), "ORIGINAL-B").unwrap();

    let a = item(&dir, "a.pak", "CZ-A");
    let new_file = item(&dir, "new.pak", "CZ-NEW");
    let from_b = dir.join("src/b.pak");
    fs::write(&from_b, "CZ-B").unwrap();
    let b = InstallItem {
        from: from_b.to_str().unwrap().into(),
        to: locked_dir.join("b.pak").to_str().unwrap().into(),
        own: false,
    };
    let items = [a, new_file, b];

    stage_files(&items).unwrap();
    // the last file can't be renamed, like a .pak held by a running game
    fs::set_permissions(&locked_dir, fs::Permissions::from_mode(0o555)).unwrap();
    let result = commit_files(&items);
    fs::set_permissions(&locked_dir, fs::Permissions::from_mode(0o755)).unwrap();

    let error = result.unwrap_err();
    assert!(error.to.ends_with("b.pak"));
    assert!(error.rollback_failed.is_empty());
    assert_eq!(read(&dir, "a.pak"), "ORIGINAL-A");
    assert_eq!(read(&dir, "locked/b.pak"), "ORIGINAL-B");
    assert_eq!(game_files(&dir), ["a.pak", "locked"]);
}

#[test]
fn stage_failure_touches_no_game_file() {
    let dir = temp_dir("stage");
    fs::write(dir.join("game/a.pak"), "ORIGINAL").unwrap();
    let a = item(&dir, "a.pak", "CZ");
    let missing = InstallItem {
        from: dir.join("src/missing").to_str().unwrap().into(),
        to: dir.join("game/b.pak").to_str().unwrap().into(),
        own: false,
    };

    assert!(install(&[a, missing]).is_err());
    assert_eq!(read(&dir, "a.pak"), "ORIGINAL");
    assert_eq!(game_files(&dir), ["a.pak"]);
}

#[test]
fn leftover_of_a_killed_install_is_not_lost() {
    let dir = temp_dir("leftover");
    fs::write(dir.join("game/a.pak"), "HALF-DONE").unwrap();
    fs::write(dir.join("game/a.pak.spyker-old"), "ORIGINAL").unwrap();

    install(&[item(&dir, "a.pak", "CZ")]).unwrap();
    assert_eq!(read(&dir, "a.pak"), "CZ");
    assert_eq!(read(&dir, "a.pak.backup"), "ORIGINAL");
    assert_eq!(game_files(&dir), ["a.pak", "a.pak.backup"]);
}
