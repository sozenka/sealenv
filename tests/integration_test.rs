use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn sealenv_bin() -> String {
    std::env::var("CARGO_BIN_EXE_sealenv").expect("CARGO_BIN_EXE_sealenv not set")
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock error")
        .as_nanos();
    let pid = std::process::id();
    std::env::temp_dir().join(format!("sealenv-{prefix}-{pid}-{now}"))
}

fn run_sealenv(project_dir: &PathBuf, home_dir: &PathBuf, args: &[&str]) -> Output {
    Command::new(sealenv_bin())
        .args(args)
        .current_dir(project_dir)
        .env("HOME", home_dir)
        .env("USERPROFILE", home_dir)
        .output()
        .expect("failed to run sealenv")
}

fn read_project_id(project_dir: &PathBuf) -> String {
    let cfg = fs::read_to_string(project_dir.join(".sealenv").join("config.toml"))
        .expect("read config.toml failed");

    for line in cfg.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("project_id") {
            let parts: Vec<&str> = trimmed.split('"').collect();
            if parts.len() >= 2 {
                return parts[1].to_string();
            }
        }
    }

    panic!("project_id not found in config.toml")
}

#[test]
fn test_help_runs() {
    let output = Command::new(sealenv_bin())
        .arg("--help")
        .output()
        .expect("failed to run --help");
    assert!(output.status.success());
}

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let project_dir = unique_test_dir("roundtrip-project");
    let home_dir = unique_test_dir("roundtrip-home");
    fs::create_dir_all(&project_dir).expect("create project dir failed");
    fs::create_dir_all(&home_dir).expect("create home dir failed");

    let original = "API_KEY=hello\nDATABASE_URL=postgres://localhost/db\n";
    fs::write(project_dir.join(".env"), original).expect("write .env failed");

    let init = run_sealenv(&project_dir, &home_dir, &["init"]);
    assert!(init.status.success(), "init failed: {}", String::from_utf8_lossy(&init.stderr));

    let encrypt = run_sealenv(&project_dir, &home_dir, &["encrypt"]);
    assert!(
        encrypt.status.success(),
        "encrypt failed: {}",
        String::from_utf8_lossy(&encrypt.stderr)
    );
    assert!(project_dir.join(".env.enc").exists(), ".env.enc was not created");

    fs::write(project_dir.join(".env"), "CORRUPTED=1\n").expect("overwrite .env failed");

    let decrypt = run_sealenv(&project_dir, &home_dir, &["decrypt"]);
    assert!(
        decrypt.status.success(),
        "decrypt failed: {}",
        String::from_utf8_lossy(&decrypt.stderr)
    );

    let restored = fs::read_to_string(project_dir.join(".env")).expect("read .env failed");
    assert_eq!(restored, original);

    let _ = fs::remove_dir_all(&project_dir);
    let _ = fs::remove_dir_all(&home_dir);
}

#[test]
fn test_decrypt_fails_with_wrong_key_home() {
    let project_dir = unique_test_dir("wrong-key-project");
    let home_a = unique_test_dir("wrong-key-home-a");
    fs::create_dir_all(&project_dir).expect("create project dir failed");
    fs::create_dir_all(&home_a).expect("create home_a failed");

    fs::write(project_dir.join(".env"), "API_KEY=hello\n").expect("write .env failed");

    let init = run_sealenv(&project_dir, &home_a, &["init"]);
    assert!(init.status.success(), "init failed: {}", String::from_utf8_lossy(&init.stderr));

    let encrypt = run_sealenv(&project_dir, &home_a, &["encrypt"]);
    assert!(
        encrypt.status.success(),
        "encrypt failed: {}",
        String::from_utf8_lossy(&encrypt.stderr)
    );

    // Overwrite the key with different bytes to simulate wrong-key decrypt attempts.
    let project_id = read_project_id(&project_dir);
    let key_path = dirs::home_dir()
        .expect("home directory not found")
        .join(".sealenv")
        .join("keys")
        .join(format!("{}.key", project_id));
    fs::write(&key_path, [7u8; 32]).expect("failed to overwrite key");

    let decrypt_wrong = run_sealenv(&project_dir, &home_a, &["decrypt"]);
    assert!(!decrypt_wrong.status.success(), "decrypt unexpectedly succeeded with wrong home key");

    let _ = fs::remove_dir_all(&project_dir);
    let _ = fs::remove_dir_all(&home_a);
}

#[test]
fn test_profile_create_and_switch() {
    let project_dir = unique_test_dir("profile-project");
    let home_dir = unique_test_dir("profile-home");
    fs::create_dir_all(&project_dir).expect("create project dir failed");
    fs::create_dir_all(&home_dir).expect("create home dir failed");

    fs::write(project_dir.join(".env"), "API_KEY=dev\n").expect("write .env failed");

    let init = run_sealenv(&project_dir, &home_dir, &["init"]);
    assert!(init.status.success(), "init failed: {}", String::from_utf8_lossy(&init.stderr));

    let create_dev = run_sealenv(&project_dir, &home_dir, &["profile", "create", "dev"]);
    assert!(
        create_dev.status.success(),
        "create dev failed: {}",
        String::from_utf8_lossy(&create_dev.stderr)
    );

    fs::write(project_dir.join(".env"), "API_KEY=staging\n").expect("write staging .env failed");

    let create_staging = run_sealenv(&project_dir, &home_dir, &["profile", "create", "staging"]);
    assert!(
        create_staging.status.success(),
        "create staging failed: {}",
        String::from_utf8_lossy(&create_staging.stderr)
    );

    let use_dev = run_sealenv(&project_dir, &home_dir, &["profile", "use", "dev"]);
    assert!(use_dev.status.success(), "use dev failed: {}", String::from_utf8_lossy(&use_dev.stderr));
    let dev_env = fs::read_to_string(project_dir.join(".env")).expect("read dev .env failed");
    assert_eq!(dev_env, "API_KEY=dev\n");

    let use_staging = run_sealenv(&project_dir, &home_dir, &["profile", "use", "staging"]);
    assert!(
        use_staging.status.success(),
        "use staging failed: {}",
        String::from_utf8_lossy(&use_staging.stderr)
    );
    let staging_env = fs::read_to_string(project_dir.join(".env")).expect("read staging .env failed");
    assert_eq!(staging_env, "API_KEY=staging\n");

    let _ = fs::remove_dir_all(&project_dir);
    let _ = fs::remove_dir_all(&home_dir);
}
