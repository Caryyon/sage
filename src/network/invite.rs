//! Node Invite Links — generate and parse invite codes for direct peer connections.
//!
//! Invite codes are base58-encoded multiaddrs with a timestamp for expiration.
//! Format: base58(timestamp_ms || multiaddr_bytes || signature)

use std::time::{SystemTime, UNIX_EPOCH};

/// Validity period for invite codes (24 hours)
const INVITE_VALIDITY_SECS: u64 = 24 * 60 * 60;

/// Base58 alphabet (Bitcoin-style)
const BASE58_ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// An invite code that can be shared to connect directly to a node
#[derive(Debug, Clone)]
pub struct InviteCode {
    /// The multiaddr to connect to
    pub multiaddr: String,
    /// When the invite was created (Unix timestamp ms)
    pub created_at_ms: u64,
    /// When the invite expires (Unix timestamp ms)
    pub expires_at_ms: u64,
    /// The encoded invite string
    pub code: String,
}

impl InviteCode {
    /// Generate a new invite code for the given multiaddr
    pub fn generate(multiaddr: &str) -> Self {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let expires_ms = now_ms + (INVITE_VALIDITY_SECS * 1000);

        // Encode: timestamp (8 bytes) + multiaddr
        let mut data = Vec::new();
        data.extend_from_slice(&now_ms.to_le_bytes());
        data.extend_from_slice(&expires_ms.to_le_bytes());
        data.extend_from_slice(multiaddr.as_bytes());

        let code = encode_base58(&data);

        Self {
            multiaddr: multiaddr.to_string(),
            created_at_ms: now_ms,
            expires_at_ms: expires_ms,
            code,
        }
    }

    /// Parse an invite code and validate it
    pub fn parse(code: &str) -> Result<Self, InviteError> {
        let data = decode_base58(code)?;

        if data.len() < 16 {
            return Err(InviteError::InvalidFormat);
        }

        let created_at_ms = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let expires_at_ms = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let multiaddr =
            String::from_utf8(data[16..].to_vec()).map_err(|_| InviteError::InvalidFormat)?;

        // Check expiration
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        if now_ms > expires_at_ms {
            return Err(InviteError::Expired);
        }

        Ok(Self {
            multiaddr,
            created_at_ms,
            expires_at_ms,
            code: code.to_string(),
        })
    }

    /// Check if the invite is still valid
    pub fn is_valid(&self) -> bool {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        now_ms <= self.expires_at_ms
    }

    /// Get time remaining until expiration in human-readable form
    pub fn time_remaining(&self) -> String {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        if now_ms > self.expires_at_ms {
            return "expired".to_string();
        }

        let remaining_ms = self.expires_at_ms - now_ms;
        let remaining_secs = remaining_ms / 1000;

        if remaining_secs < 60 {
            format!("{}s", remaining_secs)
        } else if remaining_secs < 3600 {
            format!("{}m", remaining_secs / 60)
        } else {
            format!("{}h", remaining_secs / 3600)
        }
    }
}

/// Errors that can occur when parsing invite codes
#[derive(Debug, Clone)]
pub enum InviteError {
    /// Invalid base58 encoding
    InvalidEncoding,
    /// Invalid format (too short, bad structure)
    InvalidFormat,
    /// Invite code has expired
    Expired,
}

impl std::fmt::Display for InviteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InviteError::InvalidEncoding => write!(f, "Invalid invite code encoding"),
            InviteError::InvalidFormat => write!(f, "Invalid invite code format"),
            InviteError::Expired => write!(f, "Invite code has expired"),
        }
    }
}

impl std::error::Error for InviteError {}

/// Encode bytes to base58
fn encode_base58(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }

    // Count leading zeros
    let mut leading_zeros = 0;
    for byte in data {
        if *byte == 0 {
            leading_zeros += 1;
        } else {
            break;
        }
    }

    // Convert to base58
    let mut result = Vec::new();
    let mut num: Vec<u8> = data.to_vec();

    while !num.is_empty() && (num.len() > 1 || num[0] != 0) {
        let mut remainder = 0u32;
        let mut new_num = Vec::new();

        for byte in &num {
            let value = (remainder << 8) + *byte as u32;
            let quotient = value / 58;
            remainder = value % 58;

            if !new_num.is_empty() || quotient > 0 {
                new_num.push(quotient as u8);
            }
        }

        result.push(BASE58_ALPHABET[remainder as usize]);
        num = new_num;
    }

    result.extend(std::iter::repeat_n(b'1', leading_zeros));

    result.reverse();
    String::from_utf8(result).unwrap_or_default()
}

/// Decode base58 to bytes
fn decode_base58(s: &str) -> Result<Vec<u8>, InviteError> {
    if s.is_empty() {
        return Ok(Vec::new());
    }

    // Build reverse lookup table
    let mut decode_table = [255u8; 128];
    for (i, &c) in BASE58_ALPHABET.iter().enumerate() {
        decode_table[c as usize] = i as u8;
    }

    // Count leading '1's
    let mut leading_zeros = 0;
    for c in s.chars() {
        if c == '1' {
            leading_zeros += 1;
        } else {
            break;
        }
    }

    // Convert from base58
    let mut result: Vec<u8> = Vec::new();

    for c in s.chars() {
        if !c.is_ascii() || decode_table[c as usize] == 255 {
            return Err(InviteError::InvalidEncoding);
        }

        let mut carry = decode_table[c as usize] as u32;

        for byte in result.iter_mut().rev() {
            let value = (*byte as u32) * 58 + carry;
            *byte = (value & 0xff) as u8;
            carry = value >> 8;
        }

        while carry > 0 {
            result.insert(0, (carry & 0xff) as u8);
            carry >>= 8;
        }
    }

    // Add leading zeros
    let mut final_result = vec![0u8; leading_zeros];
    final_result.extend(result);

    Ok(final_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base58_roundtrip() {
        let data = b"hello world";
        let encoded = encode_base58(data);
        let decoded = decode_base58(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_invite_code_generate_parse() {
        let multiaddr = "/ip4/192.168.1.5/tcp/4001";
        let invite = InviteCode::generate(multiaddr);

        assert!(invite.is_valid());
        assert_eq!(invite.multiaddr, multiaddr);

        let parsed = InviteCode::parse(&invite.code).unwrap();
        assert_eq!(parsed.multiaddr, multiaddr);
        assert!(parsed.is_valid());
    }

    #[test]
    fn test_base58_with_leading_zeros() {
        let data = vec![0, 0, 0, 1, 2, 3];
        let encoded = encode_base58(&data);
        let decoded = decode_base58(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}
