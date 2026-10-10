//! Clipboard output with a single, platform-specific backend per operating system.

#[cfg(any(all(unix, not(target_os = "macos")), not(any(unix, windows))))]
use anyhow::bail;
use anyhow::{Context, Result};

#[cfg(unix)]
use std::io::Write;
#[cfg(unix)]
use std::process::{Command, Stdio};

#[cfg(unix)]
fn pipe_to_command(program: &str, args: &[&str], payload: &str) -> Result<()> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("could not start {program}"))?;
    let mut stdin = child
        .stdin
        .take()
        .context("clipboard process has no stdin")?;
    stdin
        .write_all(payload.as_bytes())
        .with_context(|| format!("could not send text to {program}"))?;
    drop(stdin);
    let status = child
        .wait()
        .with_context(|| format!("could not wait for {program}"))?;
    if !status.success() {
        anyhow::bail!("{program} exited with status {status}");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn write_clipboard(payload: &str) -> Result<()> {
    pipe_to_command("pbcopy", &[], payload)
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn write_clipboard(payload: &str) -> Result<()> {
    // Prefer the active desktop session. All of these utilities persist
    // clipboard ownership after the parent CLI process exits.
    let candidates: &[(&str, &[&str])] = if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        &[
            ("wl-copy", &[]),
            ("xclip", &["-selection", "clipboard", "-in"]),
            ("xsel", &["--clipboard", "--input"]),
        ]
    } else {
        &[
            ("xclip", &["-selection", "clipboard", "-in"]),
            ("xsel", &["--clipboard", "--input"]),
            ("wl-copy", &[]),
        ]
    };
    let mut attempts = Vec::new();
    for (program, args) in candidates {
        match pipe_to_command(program, args, payload) {
            Ok(()) => return Ok(()),
            Err(error) => attempts.push(format!("{program}: {error:#}")),
        }
    }
    bail!(
        "no working desktop clipboard backend; install wl-clipboard (Wayland) or xclip/xsel (X11), or use --stdout. Attempts: {}",
        attempts.join("; ")
    )
}

#[cfg(windows)]
pub fn write_clipboard(payload: &str) -> Result<()> {
    clipboard_win::set_clipboard_string(payload).context("could not write Windows clipboard")
}

#[cfg(not(any(unix, windows)))]
pub fn write_clipboard(_payload: &str) -> Result<()> {
    bail!("no clipboard implementation for this platform; use --stdout")
}
