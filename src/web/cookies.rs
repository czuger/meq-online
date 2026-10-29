//! Minimal cookie handling: reading the `Cookie` header and building `Set-Cookie` values.

use axum::http::HeaderMap;
use axum::http::header::COOKIE;

/// Name of the session cookie.
pub const SESSION_COOKIE: &str = "meq_session";

/// Name of the flash message cookie.
pub const FLASH_COOKIE: &str = "meq_flash";

/// Value of the cookie `name` sent by the browser.
pub fn read_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.to_owned())
}

/// `Set-Cookie` value storing `value` (already cookie-safe) for the whole site.
pub fn set_cookie(name: &str, value: &str) -> String {
    format!("{name}={value}; Path=/; HttpOnly; SameSite=Lax")
}

/// `Set-Cookie` value deleting a cookie.
pub fn clear_cookie(name: &str) -> String {
    format!("{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}

/// Percent-encodes text so it can be stored in a cookie.
pub fn encode(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                char::from(b).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}

/// Decodes text produced by [`encode`]; invalid sequences are kept as they are.
pub fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok());
        match (
            bytes.get(i),
            hex.and_then(|h| u8::from_str_radix(h, 16).ok()),
        ) {
            (Some(b'%'), Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (Some(byte), _) => {
                out.push(*byte);
                i += 1;
            }
            (None, _) => break,
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoding_round_trip() {
        let text = "Carte défaussée ; 2 = deux";
        assert_eq!(decode(&encode(text)), text);
    }

    #[test]
    fn reads_one_cookie_among_several() {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, "a=1; meq_session=abc; b=2".parse().unwrap());
        assert_eq!(
            read_cookie(&headers, SESSION_COOKIE).as_deref(),
            Some("abc")
        );
        assert_eq!(read_cookie(&headers, "missing"), None);
    }
}
