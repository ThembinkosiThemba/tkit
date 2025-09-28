use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("tkit").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("A customizable tool manager"));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("tkit").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0.1.1"));
}

#[test]
fn test_list_empty_config() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("tkit").unwrap();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join(".config"))
        .arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No tools configured"));
}

// #[test]
// fn test_add_tool_missing_description() {
//     let temp_dir = TempDir::new().unwrap();
//     let mut cmd = Command::cargo_bin("tkit").unwrap();
//     cmd.env("HOME", temp_dir.path())
//         .env("XDG_CONFIG_HOME", temp_dir.path().join(".config"))
//         .arg("add")
//         .arg("test-tool")
//         .write_stdin("");

//     cmd.assert()
//         .failure()
//         .stderr(predicate::str::contains("Description is required"));
// }

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("tkit").unwrap();
    cmd.arg("invalid-command");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn test_install_nonexistent_tool() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("tkit").unwrap();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join(".config"))
        .arg("install")
        .arg("nonexistent");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Tool 'nonexistent' not found"));
}

#[test]
fn test_delete_nonexistent_tool() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("tkit").unwrap();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join(".config"))
        .arg("delete")
        .arg("nonexistent");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Tool 'nonexistent' not found"));
}

#[test]
fn test_run_nonexistent_tool() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("tkit").unwrap();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join(".config"))
        .arg("run")
        .arg("nonexistent");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Tool 'nonexistent' not found"));
}

// #[test]
// fn test_init_creates_config() {
//     let temp_dir = TempDir::new().unwrap();
//     let mut cmd = Command::cargo_bin("tkit").unwrap();
//     cmd.env("HOME", temp_dir.path())
//         .env("XDG_CONFIG_HOME", temp_dir.path().join(".config"))
//         .arg("init");

//     cmd.assert()
//         .success()
//         .stdout(predicate::str::contains("Configuration initialized with example tools"));

//     // Verify config was created by listing tools
//     let mut list_cmd = Command::cargo_bin("tkit").unwrap();
//     list_cmd.env("HOME", temp_dir.path())
//         .env("XDG_CONFIG_HOME", temp_dir.path().join(".config"))
//         .arg("list");

//     list_cmd.assert()
//         .success()
//         .stdout(predicate::str::contains("node"))
//         .stdout(predicate::str::contains("docker"));
// }
