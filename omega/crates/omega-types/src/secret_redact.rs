//! Masquage des secrets dans les chaînes et valeurs JSON (P2-6).
//!
//! Détecteur de patterns sensibles (`sk_live_`, `AKIA`, `Bearer`, `ghp_`,
//! blocs de clé privée, hex haute entropie 32+) remplacés par
//! `[REDACTED:<sha8>]` où `<sha8>` = 8 premiers hex de SHA-256 du secret
//! (stable : même secret → même tag, sans révéler le secret).

use serde_json::Value;
use sha2::{Digest, Sha256};

/// Calcule le tag stable d'un secret.
pub fn secret_tag(secret: &str) -> String {
    let mut h = Sha256::new();
    h.update(secret.as_bytes());
    let r = h.finalize();
    let hex = format!("{:x}", r);
    format!("[REDACTED:{}]", &hex[..8])
}

/// Masque les secrets dans une chaîne. Le texte normal est laissé intact.
pub fn redact_secrets(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let n = input.len();
    while i < n {
        // Trouve le plus long motif commençant à i.
        if let Some((end, secret)) = detect_at(input, i) {
            out.push_str(&secret_tag(&secret));
            i = end;
        } else {
            // Avance d'un caractère (UTF-8 safe).
            let ch = input[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

/// Détecte un secret commençant à la position `i`. Renvoie (fin exclusive, secret).
fn detect_at(input: &str, i: usize) -> Option<(usize, String)> {
    let rest = &input[i..];

    // Bloc de clé privée.
    if rest.starts_with("-----BEGIN") {
        // Uniquement les blocs de clé privée.
        if rest.contains("PRIVATE KEY") {
            if let Some(end_marker) = rest.find("-----END") {
                if let Some(rel) = rest[end_marker + 5..].find("-----") {
                    let close = end_marker + 5 + rel + 5;
                    let block = &rest[..close];
                    return Some((i + close, block.to_string()));
                }
            }
        }
    }

    // sk_live_
    if let Some(stripped) = rest.strip_prefix("sk_live_") {
        let secret_end = take_hex(stripped, 16);
        if secret_end >= 16 {
            let total = 8 + secret_end;
            return Some((i + total, rest[..total].to_string()));
        }
    }

    // AKIA (AWS)
    if let Some(stripped) = rest.strip_prefix("AKIA") {
        let secret_end = take_alnum_upper(stripped, 16);
        if secret_end >= 16 {
            let total = 4 + secret_end;
            return Some((i + total, rest[..total].to_string()));
        }
    }

    // ghp_ (GitHub PAT)
    if let Some(stripped) = rest.strip_prefix("ghp_") {
        let secret_end = take_alnum(stripped);
        if secret_end >= 36 {
            let total = 4 + secret_end;
            return Some((i + total, rest[..total].to_string()));
        }
    }

    // Bearer token
    if rest.starts_with("Bearer ") || rest.starts_with("bearer ") {
        let prefix_len = 7; // "Bearer " / "bearer "
        let token_start = prefix_len;
        // token = run of non-whitespace
        let token_bytes = &rest[token_start..];
        let token_len = token_bytes
            .find(|c: char| c.is_whitespace())
            .unwrap_or(token_bytes.len());
        if token_len > 0 {
            let token = &token_bytes[..token_len];
            // only redact if token looks like a secret (has some length)
            if token.len() >= 8 {
                let total = token_start + token_len;
                return Some((i + total, token.to_string()));
            }
        }
    }

    // Haute entropie hex 32+
    if is_hex_byte(rest.as_bytes().first().copied().unwrap_or(0)) {
        let hex_len = rest.bytes().take_while(|b| is_hex_byte(*b)).count();
        if hex_len >= 32 {
            let secret = &rest[..hex_len];
            return Some((i + hex_len, secret.to_string()));
        }
    }

    None
}

fn take_hex(s: &str, _min: usize) -> usize {
    s.bytes().take_while(|b| is_hex_byte(*b)).count()
}

fn take_alnum_upper(s: &str, _min: usize) -> usize {
    s.bytes()
        .take_while(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        .count()
}

fn take_alnum(s: &str) -> usize {
    s.bytes().take_while(|b| b.is_ascii_alphanumeric()).count()
}

fn is_hex_byte(b: u8) -> bool {
    b.is_ascii_hexdigit()
}

/// Masque récursivement les secrets dans une valeur JSON (chaînes).
pub fn redact_json(value: &mut Value) {
    match value {
        Value::String(s) => {
            let r = redact_secrets(s);
            *s = r;
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                redact_json(v);
            }
        }
        Value::Object(map) => {
            for v in map.values_mut() {
                redact_json(v);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_text_unchanged() {
        let s = "hello world, no secrets here";
        assert_eq!(redact_secrets(s), s);
    }

    #[test]
    fn sk_live_redacted() {
        let s = "key=sk_live_0123456789abcdef";
        let r = redact_secrets(s);
        assert!(!r.contains("sk_live_"));
        assert!(r.starts_with("key="));
        assert!(r.contains("[REDACTED:"));
    }

    #[test]
    fn stable_tag() {
        let a = redact_secrets("sk_live_0123456789abcdef");
        let b = redact_secrets("sk_live_0123456789abcdef");
        assert_eq!(a, b);
    }
}
