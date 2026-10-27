//! Import accounts from `otpauth://` and Google Authenticator `otpauth-migration://` URIs.

use crate::otp::{Account, Algorithm};
use base64::Engine;

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("unsupported or malformed URI")]
    BadUri,
    #[error("invalid base64/base32 data")]
    BadEncoding,
    #[error("malformed migration payload")]
    BadPayload,
    #[error("HOTP (counter-based) accounts are not supported yet: {0}")]
    HotpUnsupported(String),
}

pub fn import_uri(uri: &str) -> Result<Vec<Account>, ImportError> {
    let uri = uri.trim();
    if let Some(rest) = uri.strip_prefix("otpauth-migration://offline?") {
        parse_migration(rest)
    } else if uri.starts_with("otpauth://") {
        parse_otpauth(uri).map(|a| vec![a])
    } else {
        Err(ImportError::BadUri)
    }
}

fn query_param<'a>(query: &'a str, key: &str) -> Option<String> {
    query.split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        (k == key).then(|| urlencoding::decode(v).ok().map(|c| c.into_owned()))?
    })
}

fn parse_otpauth(uri: &str) -> Result<Account, ImportError> {
    let rest = uri.strip_prefix("otpauth://").ok_or(ImportError::BadUri)?;
    let (kind_label, query) = rest.split_once('?').ok_or(ImportError::BadUri)?;
    let (kind, label) = kind_label.split_once('/').ok_or(ImportError::BadUri)?;
    let label = urlencoding::decode(label).map_err(|_| ImportError::BadUri)?.into_owned();
    if kind != "totp" {
        return Err(ImportError::HotpUnsupported(label));
    }
    let (label_issuer, name) = match label.split_once(':') {
        Some((i, n)) => (i.trim().to_string(), n.trim().to_string()),
        None => (String::new(), label.clone()),
    };
    let issuer = query_param(query, "issuer").unwrap_or(label_issuer);
    let secret_b32 = query_param(query, "secret").ok_or(ImportError::BadUri)?;
    let secret = base32::decode(
        base32::Alphabet::Rfc4648 { padding: false },
        &secret_b32.replace(' ', "").trim_end_matches('=').to_uppercase(),
    )
    .ok_or(ImportError::BadEncoding)?;
    let algorithm = match query_param(query, "algorithm").as_deref().map(str::to_uppercase).as_deref() {
        None | Some("SHA1") => Algorithm::Sha1,
        Some("SHA256") => Algorithm::Sha256,
        Some("SHA512") => Algorithm::Sha512,
        _ => return Err(ImportError::BadUri),
    };
    Ok(Account {
        issuer,
        name,
        secret,
        algorithm,
        digits: query_param(query, "digits").and_then(|d| d.parse().ok()).unwrap_or(6),
        period: query_param(query, "period").and_then(|d| d.parse().ok()).unwrap_or(30),
    })
}

// --- minimal protobuf reader (only what MigrationPayload needs) ---

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn varint(&mut self) -> Result<u64, ImportError> {
        let mut v = 0u64;
        for shift in (0..64).step_by(7) {
            let (&b, rest) = self.0.split_first().ok_or(ImportError::BadPayload)?;
            self.0 = rest;
            v |= u64::from(b & 0x7f) << shift;
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
        Err(ImportError::BadPayload)
    }

    /// Returns (field number, wire value) where length-delimited values are slices.
    fn next(&mut self) -> Result<Option<(u64, Field<'a>)>, ImportError> {
        if self.0.is_empty() {
            return Ok(None);
        }
        let tag = self.varint()?;
        let field = match tag & 7 {
            0 => Field::Varint(self.varint()?),
            2 => {
                let len = self.varint()? as usize;
                if len > self.0.len() {
                    return Err(ImportError::BadPayload);
                }
                let (data, rest) = self.0.split_at(len);
                self.0 = rest;
                Field::Bytes(data)
            }
            _ => return Err(ImportError::BadPayload),
        };
        Ok(Some((tag >> 3, field)))
    }
}

enum Field<'a> {
    Varint(u64),
    Bytes(&'a [u8]),
}

fn parse_migration(query: &str) -> Result<Vec<Account>, ImportError> {
    let data = query_param(query, "data").ok_or(ImportError::BadUri)?;
    let raw = base64::engine::general_purpose::STANDARD
        .decode(data.replace(' ', "+"))
        .map_err(|_| ImportError::BadEncoding)?;
    let mut payload = Reader(&raw);
    let mut accounts = Vec::new();
    while let Some((num, field)) = payload.next()? {
        if let (1, Field::Bytes(entry)) = (num, field) {
            accounts.push(parse_otp_parameters(entry)?);
        }
    }
    Ok(accounts)
}

fn parse_otp_parameters(buf: &[u8]) -> Result<Account, ImportError> {
    let mut r = Reader(buf);
    let (mut secret, mut name, mut issuer) = (Vec::new(), String::new(), String::new());
    let (mut algo, mut digits, mut kind) = (1, 1, 2);
    while let Some((num, field)) = r.next()? {
        match (num, field) {
            (1, Field::Bytes(b)) => secret = b.to_vec(),
            (2, Field::Bytes(b)) => name = String::from_utf8_lossy(b).into_owned(),
            (3, Field::Bytes(b)) => issuer = String::from_utf8_lossy(b).into_owned(),
            (4, Field::Varint(v)) => algo = v,
            (5, Field::Varint(v)) => digits = v,
            (6, Field::Varint(v)) => kind = v,
            _ => {}
        }
    }
    if kind == 1 {
        return Err(ImportError::HotpUnsupported(name));
    }
    // Google stores "Issuer:name" in the name field when issuer is also set.
    if let Some((_, rest)) = name.split_once(':') {
        if !issuer.is_empty() {
            name = rest.trim().to_string();
        }
    }
    Ok(Account {
        issuer,
        name,
        secret,
        algorithm: match algo {
            3 => Algorithm::Sha512,
            2 => Algorithm::Sha256,
            _ => Algorithm::Sha1,
        },
        digits: if digits == 2 { 8 } else { 6 },
        period: 30,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otpauth_uri() {
        let a = import_uri(
            "otpauth://totp/GitHub:me%40x.com?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=GitHub",
        )
        .unwrap();
        assert_eq!(a[0].issuer, "GitHub");
        assert_eq!(a[0].name, "me@x.com");
        assert_eq!(a[0].secret, b"12345678901234567890");
        assert_eq!(a[0].code_at(59).len(), 6);
    }

    #[test]
    fn migration_payload() {
        // One entry: secret "12345678901234567890", name "a@b", issuer "Acme", SHA1, 6 digits, TOTP.
        let mut entry = vec![0x0a, 20];
        entry.extend_from_slice(b"12345678901234567890");
        entry.extend_from_slice(&[0x12, 3]);
        entry.extend_from_slice(b"a@b");
        entry.extend_from_slice(&[0x1a, 4]);
        entry.extend_from_slice(b"Acme");
        entry.extend_from_slice(&[0x20, 1, 0x28, 1, 0x30, 2]);
        let mut payload = vec![0x0a, entry.len() as u8];
        payload.extend(entry);
        let b64 = base64::engine::general_purpose::STANDARD.encode(payload);
        let uri = format!("otpauth-migration://offline?data={}", urlencoding::encode(&b64));
        let a = import_uri(&uri).unwrap();
        assert_eq!(a.len(), 1);
        assert_eq!((a[0].issuer.as_str(), a[0].name.as_str()), ("Acme", "a@b"));
        assert_eq!(a[0].secret, b"12345678901234567890");
    }
}
