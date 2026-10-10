//! Black-box tests avoid the desktop clipboard and exercise the public CLI.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run_cli(directory: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cpc"))
        .current_dir(directory)
        .args(args)
        .output()
        .expect("start cpc")
}

#[test]
fn multiple_inputs_exact_clipboard_format() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("src")).unwrap();
    fs::write(temp.path().join("main.py"), "print('hi')\n").unwrap();
    fs::write(temp.path().join("src").join("util.py"), "n = 1").unwrap();

    let result = run_cli(temp.path(), &["main.py", "src", "--stdout", "--no-tokens"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "==== main.py ====\nprint('hi')\n\n==== src/util.py ====\nn = 1\n"
    );
    let report = String::from_utf8(result.stderr).unwrap();
    assert!(report.starts_with("Wrote 2 files | "));
    assert_eq!(report.lines().count(), 1);
    assert!(!report.contains("tokens"));
}

#[test]
fn directory_ignore_hidden_and_explicit_override() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("app.rs"), "fn main() {}\n").unwrap();
    fs::write(temp.path().join(".env"), "PASSWORD=example\n").unwrap();
    fs::write(temp.path().join(".gitignore"), "skip.txt\n").unwrap();
    fs::write(temp.path().join("skip.txt"), "not needed\n").unwrap();
    let result = run_cli(temp.path(), &[".", "--stdout", "--no-tokens"]);
    assert!(result.status.success());
    let content = String::from_utf8(result.stdout).unwrap();
    assert!(content.contains("app.rs"));
    assert!(!content.contains("PASSWORD"));
    assert!(!content.contains("not needed"));

    let result = run_cli(temp.path(), &[".env", "--stdout", "--no-tokens"]);
    assert!(result.status.success());
    assert!(
        String::from_utf8(result.stdout)
            .unwrap()
            .contains("PASSWORD")
    );
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("sensitive input")
    );
}

#[test]
fn no_headers_and_deduplication() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("a.txt"), "abc").unwrap();
    let result = run_cli(
        temp.path(),
        &["a.txt", "a.txt", "--stdout", "--no-headers", "--no-tokens"],
    );
    assert!(result.status.success());
    assert_eq!(result.stdout, b"abc\n");
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("Wrote 1 file")
    );
}

#[test]
fn config_override_and_invalid_keys() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("code.txt"), "hello").unwrap();
    fs::write(
        temp.path().join("cpc.toml"),
        "[output]\nheaders = false\n[report]\ntokenizer = 'off'\n",
    )
    .unwrap();
    let result = run_cli(
        temp.path(),
        &["--config", "cpc.toml", "code.txt", "--stdout"],
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.stdout, b"hello\n");
    assert!(!String::from_utf8(result.stderr).unwrap().contains("tokens"));

    fs::write(temp.path().join("bad.toml"), "[output]\nheadres=true\n").unwrap();
    let bad = run_cli(
        temp.path(),
        &["code.txt", "--config", "bad.toml", "--stdout"],
    );
    assert!(!bad.status.success());
    assert!(bad.stdout.is_empty());
}

#[test]
fn both_tokenisers_are_available() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("code.txt"), "hello").unwrap();
    for encoding in ["o200k_base", "cl100k_base"] {
        let result = run_cli(
            temp.path(),
            &[
                "code.txt",
                "--no-headers",
                "--stdout",
                "--tokenizer",
                encoding,
            ],
        );
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(
            String::from_utf8(result.stderr)
                .unwrap()
                .contains(&format!("tokens ({encoding})"))
        );
    }
}

#[test]
fn size_limit_fails_without_writing_stdout() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("big.txt"), "very large input").unwrap();
    fs::write(temp.path().join("tiny.toml"), "[limits]\nmax_bytes=5\n").unwrap();
    let result = run_cli(
        temp.path(),
        &["big.txt", "--config", "tiny.toml", "--stdout"],
    );
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("exceeds")
    );
}

#[test]
fn explicit_binary_is_an_error() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("binary.dat"), b"abc\0def").unwrap();
    let result = run_cli(temp.path(), &["binary.dat", "--stdout", "--no-tokens"]);
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("binary file")
    );
}

#[test]
fn help_runs_without_file_argument() {
    let temp = tempfile::tempdir().unwrap();
    let result = run_cli(temp.path(), &["--help"]);
    assert!(result.status.success());
    assert!(
        String::from_utf8(result.stdout)
            .unwrap()
            .contains("--no-tokens")
    );
}
