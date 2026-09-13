use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new(source: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "ketox-cli-test-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("lib.rs"), source).unwrap();
        Self { directory }
    }

    fn command(&self, subcommand: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ketox"));
        command
            .arg(subcommand)
            .arg("--source")
            .arg(self.directory.join("lib.rs"))
            .args([
                "--package",
                "dev.ketox.test",
                "--class",
                "RustApi",
                "--library",
                "ketox_test",
            ]);
        command
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // This absolute directory was uniquely created by this fixture.
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn generate_and_inspect_agree_and_repeat_generation_is_identical() {
    let fixture = Fixture::new(
        "#[kotlin_export] pub fn hello(name: String) -> String { format!(\"Hello, {name}!\") }",
    );
    let output_directory = fixture.directory.join("generated output");
    let first = fixture
        .command("generate")
        .arg("--out")
        .arg(&output_directory)
        .output()
        .unwrap();
    success(&first);
    let first_artifacts: Vec<_> = ["RustApi.kt", "ketox_jni.rs", "ketox-metadata.json"]
        .iter()
        .map(|name| fs::read(output_directory.join(name)).unwrap())
        .collect();
    let inspection = fixture.command("inspect").output().unwrap();
    success(&inspection);
    assert_eq!(inspection.stdout, first_artifacts[2]);
    let metadata: serde_json::Value = serde_json::from_slice(&inspection.stdout).unwrap();
    assert_eq!(metadata["class_name"], "RustApi");
    assert_eq!(metadata["functions"][0]["rust_name"], "hello");
    let second = fixture
        .command("generate")
        .arg("--out")
        .arg(&output_directory)
        .output()
        .unwrap();
    success(&second);
    for (name, first) in ["RustApi.kt", "ketox_jni.rs", "ketox-metadata.json"]
        .iter()
        .zip(first_artifacts)
    {
        assert_eq!(fs::read(output_directory.join(name)).unwrap(), first);
    }
}

#[test]
fn invalid_source_does_not_create_output() {
    let fixture = Fixture::new("#[kotlin_export] pub fn invalid(value: usize) {}");
    let output_directory = fixture.directory.join("generated");
    let output = fixture
        .command("generate")
        .arg("--out")
        .arg(&output_directory)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!output_directory.exists());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("lib.rs"));
    assert!(error.contains("unsupported export parameter type"));
}

#[test]
fn invalid_command_lines_fail_with_useful_diagnostics() {
    for (arguments, expected) in [
        (vec!["unknown"], "unknown command"),
        (vec!["generate"], "--source"),
        (vec!["generate", "--source"], "missing value"),
        (vec!["generate", "--source", "--package"], "missing value"),
        (vec!["generate", "--surprise", "x"], "unknown option"),
        (
            vec!["generate", "--source", "a.rs", "--source", "b.rs"],
            "more than once",
        ),
        (vec!["inspect", "--out", "generated"], "unknown option"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_ketox"))
            .args(arguments)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected));
    }
}

#[test]
fn missing_source_and_unwritable_output_report_paths() {
    let fixture = Fixture::new("#[kotlin_export] pub fn ping() {}");
    let blocked_directory = fixture.directory.join("not-a-directory");
    fs::write(&blocked_directory, "existing user data").unwrap();
    let output = fixture
        .command("generate")
        .arg("--out")
        .arg(&blocked_directory)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not-a-directory"));
    assert_eq!(
        fs::read_to_string(&blocked_directory).unwrap(),
        "existing user data"
    );
    fs::remove_file(fixture.directory.join("lib.rs")).unwrap();
    let output = fixture.command("inspect").output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("could not read Rust source"));
}

#[test]
fn help_and_version_do_not_require_a_project() {
    for arguments in [vec!["--help"], vec!["generate", "--help"], vec![]] {
        let output = Command::new(env!("CARGO_BIN_EXE_ketox"))
            .args(arguments)
            .output()
            .unwrap();
        success(&output);
        assert!(String::from_utf8_lossy(&output.stdout).contains("Ketox"));
    }
    let output = Command::new(env!("CARGO_BIN_EXE_ketox"))
        .arg("--version")
        .output()
        .unwrap();
    success(&output);
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("ketox {}\n", env!("CARGO_PKG_VERSION"))
    );
}
