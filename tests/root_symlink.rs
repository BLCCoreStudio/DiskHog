#![cfg(unix)]

use diskhog::{scan, ScanOptions};
use std::fs;
use std::os::unix::fs::symlink;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn rejects_symbolic_link_root() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    let base =
        std::env::temp_dir().join(format!("diskhog-root-link-{}-{nonce}", std::process::id()));
    let target = base.join("target");
    let link = base.join("link");

    fs::create_dir_all(&target).expect("target directory should be created");
    fs::write(target.join("payload.bin"), vec![1_u8; 4096]).expect("fixture should be written");
    symlink(&target, &link).expect("root symlink should be created");

    let error = scan(&link, ScanOptions::default()).expect_err("root symlink must be rejected");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    assert!(error.to_string().contains("symbolic link"));

    let _ = fs::remove_dir_all(base);
}
