//! Measures precisely what is written and formats one terminal line.

use crate::bundle::Bundle;
use crate::config::Tokenizer;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    pub file_count: usize,
    pub byte_count: usize,
    pub character_count: usize,
    pub token_count: Option<usize>,
    pub tokenizer: Tokenizer,
    pub skipped_count: usize,
    pub sensitive_count: usize,
}

pub fn measure_bundle(bundle: &Bundle, tokenizer: Tokenizer) -> Result<Summary> {
    let text = &bundle.text;
    Ok(Summary {
        file_count: bundle.file_count,
        byte_count: text.len(),
        character_count: text.chars().count(),
        token_count: crate::tokens::count_tokens(text, tokenizer)?,
        tokenizer,
        skipped_count: bundle.skipped_count,
        sensitive_count: bundle.sensitive_count,
    })
}

fn grouped(value: usize) -> String {
    let digits = value.to_string();
    let mut remaining = digits.as_str();
    let mut groups = Vec::new();
    while remaining.len() > 3 {
        let split = remaining.len() - 3;
        groups.push(&remaining[split..]);
        remaining = &remaining[..split];
    }
    groups.push(remaining);
    groups.reverse();
    groups.join(",")
}

pub fn format_summary(summary: &Summary, copied: bool) -> String {
    let verb = if copied { "Copied" } else { "Wrote" };
    let noun = if summary.file_count == 1 {
        "file"
    } else {
        "files"
    };
    let mut line = format!(
        "{} {} {} | {} chars",
        verb,
        grouped(summary.file_count),
        noun,
        grouped(summary.character_count)
    );
    if let Some(count) = summary.token_count {
        line.push_str(&format!(
            " | {} tokens ({})",
            grouped(count),
            summary.tokenizer.label()
        ));
    }
    if summary.skipped_count > 0 {
        line.push_str(&format!(" | {} skipped", grouped(summary.skipped_count)));
    }
    if summary.sensitive_count > 0 {
        line.push_str(" | sensitive input");
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_is_one_line() {
        let summary = Summary {
            file_count: 38,
            byte_count: 1000,
            character_count: 983,
            token_count: Some(280),
            tokenizer: Tokenizer::O200kBase,
            skipped_count: 2,
            sensitive_count: 0,
        };
        let line = format_summary(&summary, true);
        assert_eq!(
            line,
            "Copied 38 files | 983 chars | 280 tokens (o200k_base) | 2 skipped"
        );
        assert!(!line.contains('\n'));
    }

    #[test]
    fn off_omits_token_field() {
        let summary = Summary {
            file_count: 1,
            byte_count: 5,
            character_count: 5,
            token_count: None,
            tokenizer: Tokenizer::Off,
            skipped_count: 0,
            sensitive_count: 0,
        };
        assert_eq!(format_summary(&summary, false), "Wrote 1 file | 5 chars");
    }
}
