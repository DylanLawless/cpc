//! Filesystem traversal, ignore rules, canonical deduplication and path labels.

use crate::config::Settings;
use anyhow::{Context, Result, bail};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// These restrictions apply to recursive discovery. An explicitly named file
// overrides them, which makes behaviour inspectable and unsurprising.
const DEFAULT_EXCLUDE: &[&str] = &[
    ".git",
    "node_modules",
    ".venv",
    "venv",
    "__pycache__",
    "target",
    "dist",
    ".env",
    ".env.*",
    "*.pem",
    "*.key",
    "*.p12",
    "*.pfx",
    "id_rsa",
    "id_ed25519",
    "id_ecdsa",
];

#[derive(Debug, Clone)]
pub struct SelectedFile {
    pub source_path: PathBuf,
    pub display_path: String,
    pub explicit: bool,
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub files: Vec<SelectedFile>,
}

/// Conservative filename warning for files explicitly requested by the user.
/// This is deliberately not a content-based credential scanner.
pub fn is_sensitive_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let name = name.to_ascii_lowercase();
    name == ".env"
        || name.starts_with(".env.")
        || name.starts_with("id_rsa")
        || name.starts_with("id_ed25519")
        || name.starts_with("id_ecdsa")
        || [".pem", ".key", ".p12", ".pfx"]
            .iter()
            .any(|suffix| name.ends_with(suffix))
}

fn build_exclusions(settings: &Settings) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in DEFAULT_EXCLUDE
        .iter()
        .copied()
        .chain(settings.selection.exclude.iter().map(String::as_str))
    {
        builder.add(
            Glob::new(pattern).with_context(|| format!("invalid exclude pattern {pattern:?}"))?,
        );
    }
    builder
        .build()
        .context("could not compile exclude patterns")
}

fn is_excluded(path: &Path, root: &Path, matcher: &GlobSet) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    matcher.is_match(relative)
        || path
            .file_name()
            .is_some_and(|name| matcher.is_match(Path::new(name)))
}

fn display_path(path: &Path, cwd: &Path) -> String {
    let relative = path.strip_prefix(cwd).unwrap_or(path);
    // Use forward slashes in AI-readable headers on every platform.
    let text = relative.to_string_lossy().into_owned();
    #[cfg(windows)]
    let text = text.replace('\\', "/");
    text.replace('\r', "\\r").replace('\n', "\\n")
}

fn add_file(
    path: &Path,
    explicit: bool,
    cwd: &Path,
    seen: &mut HashSet<PathBuf>,
    files: &mut Vec<SelectedFile>,
    max_files: usize,
) -> Result<()> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("cannot resolve {}", path.display()))?;
    if seen.insert(canonical.clone()) {
        if files.len() >= max_files {
            bail!("more than {max_files} selected files; increase limits.max_files in config");
        }
        files.push(SelectedFile {
            source_path: canonical,
            display_path: display_path(path, cwd),
            explicit,
        });
    }
    Ok(())
}

pub fn collect_files(input_paths: &[PathBuf], settings: &Settings) -> Result<Selection> {
    if input_paths.is_empty() {
        bail!("provide at least one file or directory");
    }
    let cwd = std::env::current_dir().context("cannot determine current directory")?;
    let exclusions = Arc::new(build_exclusions(settings)?);
    let mut seen = HashSet::new();
    let mut files = Vec::new();

    for input in input_paths {
        let metadata = std::fs::metadata(input)
            .with_context(|| format!("cannot access {}", input.display()))?;
        if metadata.is_file() {
            add_file(
                input,
                true,
                &cwd,
                &mut seen,
                &mut files,
                settings.limits.max_files,
            )?;
            continue;
        }
        if !metadata.is_dir() {
            bail!("not a regular file or directory: {}", input.display());
        }
        if std::fs::symlink_metadata(input)?.file_type().is_symlink() {
            bail!("refusing to traverse directory symlink {}", input.display());
        }

        let mut walker = WalkBuilder::new(input);
        walker.follow_links(false);
        walker.hidden(!settings.selection.include_hidden);
        walker.git_ignore(settings.selection.respect_ignore_files);
        walker.git_global(settings.selection.respect_ignore_files);
        walker.git_exclude(settings.selection.respect_ignore_files);
        walker.ignore(settings.selection.respect_ignore_files);
        walker.parents(settings.selection.respect_ignore_files);
        walker.require_git(false);
        walker.sort_by_file_path(|left, right| left.cmp(right));

        // Use a walker entry filter to prune excluded directories before descent.
        let root = input.clone();
        let matcher = Arc::clone(&exclusions);
        walker.filter_entry(move |entry| {
            entry.depth() == 0 || !is_excluded(entry.path(), &root, &matcher)
        });

        for entry in walker.build() {
            let entry = entry.with_context(|| format!("cannot walk {}", input.display()))?;
            if entry.depth() == 0 {
                continue;
            }
            if entry.file_type().is_some_and(|kind| kind.is_file()) {
                add_file(
                    entry.path(),
                    false,
                    &cwd,
                    &mut seen,
                    &mut files,
                    settings.limits.max_files,
                )?;
            }
        }
    }
    if files.is_empty() {
        bail!("no eligible files found (check ignore rules and exclusions)");
    }
    Ok(Selection { files })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn filters_directory_entries_but_not_explicit_files() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("keep.rs"), "ok").unwrap();
        fs::write(temp.path().join(".env"), "SECRET=1").unwrap();
        fs::create_dir(temp.path().join("node_modules")).unwrap();
        fs::write(temp.path().join("node_modules").join("a.js"), "skip").unwrap();
        let settings = Settings::default();
        let discovered = collect_files(&[temp.path().to_path_buf()], &settings).unwrap();
        assert_eq!(discovered.files.len(), 1);
        assert!(discovered.files[0].display_path.ends_with("keep.rs"));
        let explicit = collect_files(&[temp.path().join(".env")], &settings).unwrap();
        assert_eq!(explicit.files.len(), 1);
        assert!(explicit.files[0].explicit);
    }

    #[test]
    fn preserve_argument_order_and_deduplicate() {
        let temp = tempfile::tempdir().unwrap();
        let a = temp.path().join("a.txt");
        let b = temp.path().join("b.txt");
        fs::write(&a, "A").unwrap();
        fs::write(&b, "B").unwrap();
        let selection =
            collect_files(&[b.clone(), temp.path().into(), a], &Settings::default()).unwrap();
        assert_eq!(selection.files.len(), 2);
        assert!(selection.files[0].display_path.ends_with("b.txt"));
        assert!(selection.files[1].display_path.ends_with("a.txt"));
    }

    #[test]
    fn respects_local_gitignore() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join(".gitignore"), "ignored.txt\n").unwrap();
        fs::write(temp.path().join("ignored.txt"), "skip").unwrap();
        fs::write(temp.path().join("included.txt"), "use").unwrap();
        let selection = collect_files(&[temp.path().into()], &Settings::default()).unwrap();
        assert_eq!(selection.files.len(), 1);
        assert!(selection.files[0].display_path.ends_with("included.txt"));
    }
}
