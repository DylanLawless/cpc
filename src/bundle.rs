//! Strict UTF-8 payload construction. No clipboard writes occur here.

use crate::config::Settings;
use crate::selection::Selection;
use anyhow::{Context, Result, bail};
use std::io::Read;

#[derive(Debug, Clone)]
pub struct Bundle {
    pub text: String,
    pub file_count: usize,
    pub skipped_count: usize,
    pub sensitive_count: usize,
}

fn append_with_limit(target: &mut String, value: &str, limit: usize) -> Result<()> {
    if value.len() > limit.saturating_sub(target.len()) {
        bail!(
            "formatted content exceeds {} bytes; increase limits.max_bytes in config",
            limit
        );
    }
    target.push_str(value);
    Ok(())
}

fn has_binary_prefix(path: &std::path::Path) -> Result<bool> {
    let mut file =
        std::fs::File::open(path).with_context(|| format!("cannot open {}", path.display()))?;
    let mut prefix = [0_u8; 8192];
    let read = file.read(&mut prefix)?;
    Ok(prefix[..read].contains(&0))
}

pub fn build_bundle(selection: &Selection, settings: &Settings) -> Result<Bundle> {
    let limit = settings.limits.max_bytes;
    let mut payload = String::new();
    let mut file_count = 0;
    let mut skipped_count = 0;
    let mut sensitive_count = 0;

    for file in &selection.files {
        let metadata = std::fs::metadata(&file.source_path)
            .with_context(|| format!("cannot stat {}", file.display_path))?;
        if !metadata.is_file() {
            bail!(
                "file changed or is no longer regular: {}",
                file.display_path
            );
        }
        // Reject obviously oversized text without allocating it. An oversized
        // binary entry discovered recursively may still be skipped.
        let remaining = limit.saturating_sub(payload.len());
        if metadata.len() > remaining as u64 {
            if !file.explicit && has_binary_prefix(&file.source_path)? {
                skipped_count += 1;
                continue;
            }
            bail!("content exceeds {} bytes at {}", limit, file.display_path);
        }

        let bytes = std::fs::read(&file.source_path)
            .with_context(|| format!("cannot read {}", file.display_path))?;
        let text = if bytes.contains(&0) {
            if file.explicit {
                bail!("binary file explicitly selected: {}", file.display_path);
            }
            skipped_count += 1;
            continue;
        } else {
            match String::from_utf8(bytes) {
                Ok(text) => text,
                Err(_) if !file.explicit => {
                    skipped_count += 1;
                    continue;
                }
                Err(_) => bail!("file is not UTF-8: {}", file.display_path),
            }
        };
        if settings.output.headers {
            let header = format!("==== {} ====\n", file.display_path);
            append_with_limit(&mut payload, &header, limit)?;
        }
        append_with_limit(&mut payload, &text, limit)?;
        append_with_limit(&mut payload, "\n", limit)?;
        file_count += 1;
        if file.explicit && crate::selection::is_sensitive_file(&file.source_path) {
            sensitive_count += 1;
        }
    }
    if file_count == 0 {
        bail!("no text files available to copy");
    }
    Ok(Bundle {
        text: payload,
        file_count,
        skipped_count,
        sensitive_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::{SelectedFile, Selection};
    use std::fs;

    #[test]
    fn exact_existing_header_format() {
        let temp = tempfile::tempdir().unwrap();
        let a = temp.path().join("a.rs");
        let b = temp.path().join("b.rs");
        fs::write(&a, "let a = 1;\n").unwrap();
        fs::write(&b, "let b = 2;").unwrap();
        let selection = Selection {
            files: vec![
                SelectedFile {
                    source_path: a,
                    display_path: "a.rs".into(),
                    explicit: true,
                },
                SelectedFile {
                    source_path: b,
                    display_path: "b.rs".into(),
                    explicit: true,
                },
            ],
        };
        let result = build_bundle(&selection, &Settings::default()).unwrap();
        assert_eq!(
            result.text,
            "==== a.rs ====\nlet a = 1;\n\n==== b.rs ====\nlet b = 2;\n"
        );
        assert_eq!(result.file_count, 2);
    }

    #[test]
    fn no_headers_and_unicode() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("unicode.txt");
        fs::write(&path, "café\n").unwrap();
        let mut settings = Settings::default();
        settings.output.headers = false;
        let selection = Selection {
            files: vec![SelectedFile {
                source_path: path,
                display_path: "unicode.txt".into(),
                explicit: true,
            }],
        };
        assert_eq!(
            build_bundle(&selection, &settings).unwrap().text,
            "café\n\n"
        );
    }

    #[test]
    fn discovered_binary_is_skipped() {
        let temp = tempfile::tempdir().unwrap();
        let a = temp.path().join("file.rs");
        let b = temp.path().join("image.bin");
        fs::write(&a, "hi").unwrap();
        fs::write(&b, b"a\0b").unwrap();
        let selection = Selection {
            files: vec![
                SelectedFile {
                    source_path: b,
                    display_path: "image.bin".into(),
                    explicit: false,
                },
                SelectedFile {
                    source_path: a,
                    display_path: "file.rs".into(),
                    explicit: false,
                },
            ],
        };
        let bundle = build_bundle(&selection, &Settings::default()).unwrap();
        assert_eq!(bundle.skipped_count, 1);
        assert_eq!(bundle.file_count, 1);
    }

    #[test]
    fn limit_includes_headers() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("a.txt");
        fs::write(&path, "123").unwrap();
        let selection = Selection {
            files: vec![SelectedFile {
                source_path: path,
                display_path: "a.txt".into(),
                explicit: true,
            }],
        };
        let mut settings = Settings::default();
        settings.limits.max_bytes = 3;
        assert!(build_bundle(&selection, &settings).is_err());
    }
}
