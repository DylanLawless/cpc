//! Local token counting. Encodings are chosen explicitly, never inferred.

use crate::config::Tokenizer;
use anyhow::{Context, Result};

pub fn count_tokens(payload: &str, tokenizer: Tokenizer) -> Result<Option<usize>> {
    let encoding = match tokenizer {
        Tokenizer::Off => return Ok(None),
        Tokenizer::O200kBase => {
            tiktoken_rs::o200k_base().context("could not initialise the o200k_base encoding")?
        }
        Tokenizer::Cl100kBase => {
            tiktoken_rs::cl100k_base().context("could not initialise the cl100k_base encoding")?
        }
    };
    // Ordinary encoding treats special-token-looking text as literal input.
    // The measurement is for the exact clipboard payload, not API chat framing.
    Ok(Some(encoding.encode_ordinary(payload).len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_encodings_have_known_simple_vectors() {
        for encoding in [Tokenizer::O200kBase, Tokenizer::Cl100kBase] {
            assert_eq!(count_tokens("", encoding).unwrap(), Some(0));
            assert_eq!(count_tokens("hello", encoding).unwrap(), Some(1));
        }
    }

    #[test]
    fn off_has_no_count() {
        assert_eq!(count_tokens("any text", Tokenizer::Off).unwrap(), None);
    }
}
