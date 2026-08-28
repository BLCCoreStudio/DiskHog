use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("diskhog-cli-test-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&path).expect("test directory should be created");
        Self(path)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn help_exits_successfully() {
    let output = Command::new(env!("CARGO_BIN_EXE_diskhog"))
        .arg("--help")
        .output()
        .expect("diskhog should start");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("USAGE:"));
    assert!(stdout.contains("--limit"));
    assert!(stdout.contains("--depth"));
}

#[test]
fn scan_lists_created_file() {
    let root = TestDir::new();
    fs::write(root.0.join("visible.bin"), vec![42_u8; 8192]).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_diskhog"))
        .arg("--files")
        .arg(&root.0)
        .output()
        .expect("diskhog should start");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("visible.bin"));
}

#[test]
fn invalid_limit_exits_with_usage_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_diskhog"))
        .args(["--limit", "0"])
        .output()
        .expect("diskhog should start");

    assert_eq!(output.status.code(), Some(2));
}
