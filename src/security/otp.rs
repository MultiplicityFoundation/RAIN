use crate::config::OtpConfig;
use crate::security::secrets::SecretStore;
use anyhow::{Context, Result};
use parking_lot::Mutex;
use ring::hmac;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const OTP_SECRET_FILE: &str = "otp-secret";
const OTP_DIGITS: u32 = 6;
const OTP_ISSUER: &str = "R.A.I.N.";
/// Lockout duration after exceeding `challenge_max_attempts`.
const OTP_LOCKOUT_SECS: u64 = 300; // 5 minutes

#[derive(Debug)]
pub struct OtpValidator {
    config: OtpConfig,
    secret: Vec<u8>,
    /// Codes that have already been used — reject on re-presentation (anti-replay).
    used_codes: Mutex<HashMap<String, u64>>,
    /// Tracks consecutive failed validation attempts and optional lockout deadline.
    failed_state: Mutex<OtpFailedState>,
}

#[derive(Debug, Clone)]
struct OtpFailedState {
    count: u32,
    lockout_until: Option<u64>,
}

impl OtpValidator {
    pub fn from_config(
        config: &OtpConfig,
        rain_dir: &Path,
        store: &SecretStore,
    ) -> Result<(Self, Option<String>)> {
        let secret_path = secret_file_path(rain_dir);
        let (secret, generated) = if secret_path.exists() {
            let encoded = fs::read_to_string(&secret_path).with_context(|| {
                format!("Failed to read OTP secret file {}", secret_path.display())
            })?;
            let decrypted = store
                .decrypt(encoded.trim())
                .context("Failed to decrypt OTP secret file")?;
            (decode_base32_secret(&decrypted)?, false)
        } else {
            let raw: [u8; 20] = rand::random();
            let encoded_secret = encode_base32_secret(&raw);
            let encrypted = store
                .encrypt(&encoded_secret)
                .context("Failed to encrypt OTP secret")?;
            write_secret_file(&secret_path, &encrypted)?;
            (raw.to_vec(), true)
        };

        let validator = Self {
            config: config.clone(),
            secret,
            used_codes: Mutex::new(HashMap::new()),
            failed_state: Mutex::new(OtpFailedState {
                count: 0,
                lockout_until: None,
            }),
        };
        let uri = if generated {
            Some(validator.otpauth_uri())
        } else {
            None
        };
        Ok((validator, uri))
    }

    pub fn validate(&self, code: &str) -> Result<bool> {
        self.validate_at(code, unix_timestamp_now())
    }

    fn validate_at(&self, code: &str, now_secs: u64) -> Result<bool> {
        // Rate limiting: check lockout before any validation work.
        {
            let state = self.failed_state.lock();
            if let Some(until) = state.lockout_until {
                if now_secs < until {
                    let remaining = until.saturating_sub(now_secs);
                    anyhow::bail!(
                        "OTP locked out after {} failed attempts. Try again in {}s.",
                        self.config.challenge_max_attempts,
                        remaining
                    );
                }
            }
        }

        let normalized = code.trim();
        if normalized.len() != OTP_DIGITS as usize
            || !normalized.chars().all(|ch| ch.is_ascii_digit())
        {
            self.record_failure(now_secs);
            return Ok(false);
        }

        // Anti-replay: reject codes that have already been used within their validity window.
        {
            let mut used = self.used_codes.lock();
            used.retain(|_, expiry| *expiry >= now_secs);
            if used
                .get(normalized)
                .is_some_and(|expiry| *expiry >= now_secs)
            {
                // Replay of a valid code: reject but do NOT count toward brute-force
                // lockout. Replays are consumed codes, not wrong guesses. Counting them
                // would let an attacker lock out a legitimate user by replaying an
                // intercepted code `challenge_max_attempts` times.
                return Ok(false);
            }
        }

        let step = self.config.token_ttl_secs.max(1);
        let counter = now_secs / step;
        let counters = [
            counter.saturating_sub(1),
            counter,
            counter.saturating_add(1),
        ];

        let is_valid = counters
            .iter()
            .map(|c| compute_totp_code(&self.secret, *c))
            .any(|candidate| candidate == normalized);

        if is_valid {
            // Reset failure counter on success.
            {
                let mut state = self.failed_state.lock();
                state.count = 0;
                state.lockout_until = None;
            }
            // Mark code as used so it cannot be replayed.
            let mut used = self.used_codes.lock();
            used.insert(
                normalized.to_string(),
                now_secs.saturating_add(self.config.cache_valid_secs),
            );
        } else {
            self.record_failure(now_secs);
        }

        Ok(is_valid)
    }

    /// Record a failed attempt and trigger lockout if threshold exceeded.
    fn record_failure(&self, now_secs: u64) {
        let mut state = self.failed_state.lock();
        // If a prior lockout has expired, reset the counter.
        if state
            .lockout_until
            .is_some_and(|until| now_secs >= until)
        {
            state.count = 0;
            state.lockout_until = None;
        }
        state.count += 1;
        if state.count >= self.config.challenge_max_attempts {
            state.lockout_until = Some(now_secs.saturating_add(OTP_LOCKOUT_SECS));
            tracing::warn!(
                attempts = state.count,
                lockout_secs = OTP_LOCKOUT_SECS,
                "OTP brute-force lockout triggered"
            );
        }
    }

    pub fn otpauth_uri(&self) -> String {
        let secret = encode_base32_secret(&self.secret);
        let account = "R.A.I.N.";
        format!(
            "otpauth://totp/{issuer}:{account}?secret={secret}&issuer={issuer}&period={period}",
            issuer = OTP_ISSUER,
            period = self.config.token_ttl_secs.max(1)
        )
    }

    #[cfg(test)]
    pub(crate) fn code_for_timestamp(&self, timestamp: u64) -> String {
        let counter = timestamp / self.config.token_ttl_secs.max(1);
        compute_totp_code(&self.secret, counter)
    }
}

pub fn secret_file_path(rain_dir: &Path) -> PathBuf {
    rain_dir.join(OTP_SECRET_FILE)
}

fn write_secret_file(path: &Path, value: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    let temp_path = path.with_extension(format!("tmp-{}", uuid::Uuid::new_v4()));
    fs::write(&temp_path, value).with_context(|| {
        format!(
            "Failed to write temporary OTP secret {}",
            temp_path.display()
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o600));
    }

    fs::rename(&temp_path, path).with_context(|| {
        format!(
            "Failed to atomically replace OTP secret file {}",
            path.display()
        )
    })?;
    Ok(())
}

fn unix_timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn compute_totp_code(secret: &[u8], counter: u64) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, secret);
    let counter_bytes = counter.to_be_bytes();
    let digest = hmac::sign(&key, &counter_bytes);
    let hash = digest.as_ref();

    let offset = (hash[19] & 0x0f) as usize;
    let binary = ((u32::from(hash[offset]) & 0x7f) << 24)
        | (u32::from(hash[offset + 1]) << 16)
        | (u32::from(hash[offset + 2]) << 8)
        | u32::from(hash[offset + 3]);

    let code = binary % 10_u32.pow(OTP_DIGITS);
    format!("{code:0>6}")
}

fn encode_base32_secret(input: &[u8]) -> String {
    const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    if input.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut buffer = 0u16;
    let mut bits_left = 0u8;

    for byte in input {
        buffer = (buffer << 8) | u16::from(*byte);
        bits_left += 8;

        while bits_left >= 5 {
            let index = ((buffer >> (bits_left - 5)) & 0x1f) as usize;
            result.push(ALPHABET[index] as char);
            bits_left -= 5;
        }
    }

    if bits_left > 0 {
        let index = ((buffer << (5 - bits_left)) & 0x1f) as usize;
        result.push(ALPHABET[index] as char);
    }

    result
}

fn decode_base32_secret(raw: &str) -> Result<Vec<u8>> {
    fn decode_char(ch: char) -> Option<u8> {
        match ch {
            'A'..='Z' => Some((ch as u8) - b'A'),
            '2'..='7' => Some((ch as u8) - b'2' + 26),
            _ => None,
        }
    }

    let mut cleaned = raw
        .chars()
        .filter(|ch| !matches!(ch, ' ' | '\t' | '\n' | '\r' | '-'))
        .collect::<String>()
        .to_ascii_uppercase();
    while cleaned.ends_with('=') {
        cleaned.pop();
    }
    if cleaned.is_empty() {
        anyhow::bail!("OTP secret is empty");
    }

    let mut output = Vec::new();
    let mut buffer = 0u32;
    let mut bits_left = 0u8;

    for ch in cleaned.chars() {
        let value = decode_char(ch)
            .with_context(|| format!("OTP secret contains invalid base32 character '{ch}'"))?;
        buffer = (buffer << 5) | u32::from(value);
        bits_left += 5;

        if bits_left >= 8 {
            let byte = ((buffer >> (bits_left - 8)) & 0xff) as u8;
            output.push(byte);
            bits_left -= 8;
        }
    }

    if output.is_empty() {
        anyhow::bail!("OTP secret did not decode to any bytes");
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_config() -> OtpConfig {
        OtpConfig {
            enabled: true,
            token_ttl_secs: 30,
            cache_valid_secs: 120,
            ..OtpConfig::default()
        }
    }

    #[test]
    fn valid_totp_code_is_accepted() {
        let dir = tempdir().unwrap();
        let store = SecretStore::new(dir.path(), true);
        let (validator, _) = OtpValidator::from_config(&test_config(), dir.path(), &store).unwrap();

        let now = 1_700_000_000u64;
        let code = validator.code_for_timestamp(now);
        assert!(validator.validate_at(&code, now).unwrap());
    }

    #[test]
    fn expired_totp_code_is_rejected() {
        let dir = tempdir().unwrap();
        let store = SecretStore::new(dir.path(), true);
        let (validator, _) = OtpValidator::from_config(&test_config(), dir.path(), &store).unwrap();

        let stale = 1_700_000_000u64;
        let now = stale + 300;
        let code = validator.code_for_timestamp(stale);
        assert!(!validator.validate_at(&code, now).unwrap());
    }

    #[test]
    fn wrong_totp_code_is_rejected() {
        let dir = tempdir().unwrap();
        let store = SecretStore::new(dir.path(), true);
        let (validator, _) = OtpValidator::from_config(&test_config(), dir.path(), &store).unwrap();
        assert!(!validator.validate_at("123456", 1_700_000_000).unwrap());
    }

    #[test]
    fn lockout_after_max_failed_attempts() {
        let dir = tempdir().unwrap();
        let store = SecretStore::new(dir.path(), true);
        let mut cfg = test_config();
        cfg.challenge_max_attempts = 3;
        let (validator, _) = OtpValidator::from_config(&cfg, dir.path(), &store).unwrap();

        let now = 1_700_000_000u64;

        // Three wrong attempts should trigger lockout.
        assert!(!validator.validate_at("000000", now).unwrap());
        assert!(!validator.validate_at("000001", now).unwrap());
        assert!(!validator.validate_at("000002", now).unwrap());

        // Fourth attempt should return Err (locked out), not Ok(false).
        let err = validator.validate_at("000003", now).unwrap_err();
        assert!(
            err.to_string().contains("locked out"),
            "expected lockout error, got: {err}"
        );

        // After lockout expires, attempts work again.
        let after_lockout = now + super::OTP_LOCKOUT_SECS;
        assert!(!validator.validate_at("999999", after_lockout).unwrap());
    }

    #[test]
    fn successful_validation_resets_failure_counter() {
        let dir = tempdir().unwrap();
        let store = SecretStore::new(dir.path(), true);
        let mut cfg = test_config();
        cfg.challenge_max_attempts = 3;
        let (validator, _) = OtpValidator::from_config(&cfg, dir.path(), &store).unwrap();

        let now = 1_700_000_000u64;

        // Two failures (below threshold).
        assert!(!validator.validate_at("000000", now).unwrap());
        assert!(!validator.validate_at("000001", now).unwrap());

        // Valid code resets the counter.
        let code = validator.code_for_timestamp(now);
        assert!(validator.validate_at(&code, now).unwrap());

        // Two more failures should NOT trigger lockout (counter was reset).
        assert!(!validator.validate_at("000000", now).unwrap());
        assert!(!validator.validate_at("000001", now).unwrap());
        // Third failure after reset triggers lockout.
        assert!(!validator.validate_at("000002", now).unwrap());
        let err = validator.validate_at("000003", now).unwrap_err();
        assert!(err.to_string().contains("locked out"));
    }

    #[test]
    fn replay_of_used_code_is_rejected() {
        let dir = tempdir().unwrap();
        let store = SecretStore::new(dir.path(), true);
        let (validator, _) = OtpValidator::from_config(&test_config(), dir.path(), &store).unwrap();

        let now = 1_700_000_000u64;
        let code = validator.code_for_timestamp(now);

        // First use succeeds.
        assert!(validator.validate_at(&code, now).unwrap());
        // Immediate replay is rejected.
        assert!(!validator.validate_at(&code, now).unwrap());
        // Replay slightly later (still within cache_valid_secs) is also rejected.
        assert!(!validator.validate_at(&code, now + 10).unwrap());
    }

    #[test]
    fn replay_does_not_count_toward_lockout() {
        let dir = tempdir().unwrap();
        let store = SecretStore::new(dir.path(), true);
        let mut cfg = test_config();
        cfg.challenge_max_attempts = 3;
        let (validator, _) = OtpValidator::from_config(&cfg, dir.path(), &store).unwrap();

        let now = 1_700_000_000u64;
        let code = validator.code_for_timestamp(now);

        // Use the code once (success resets failure counter).
        assert!(validator.validate_at(&code, now).unwrap());

        // Replay the same code many times — none should trigger lockout.
        for _ in 0..10 {
            assert!(!validator.validate_at(&code, now).unwrap());
        }

        // A wrong code should still work (not locked out from replays).
        assert!(!validator.validate_at("000000", now).unwrap());
    }

    #[test]
    fn code_reusable_after_used_window_expires() {
        let dir = tempdir().unwrap();
        let store = SecretStore::new(dir.path(), true);
        let cfg = test_config(); // cache_valid_secs = 120
        let (validator, _) = OtpValidator::from_config(&cfg, dir.path(), &store).unwrap();

        let now = 1_700_000_000u64;
        let code = validator.code_for_timestamp(now);

        assert!(validator.validate_at(&code, now).unwrap());
        assert!(!validator.validate_at(&code, now).unwrap()); // replay rejected

        // After used-code window expires, but code is still TOTP-valid (within ±1 step),
        // it can be used again. With cache_valid_secs=120 and ttl=30, the used entry
        // expires at now+120, but the TOTP window is only ±30s so the code will be
        // invalid by then anyway. Verify the used entry is cleaned up.
        let after_expiry = now + cfg.cache_valid_secs + 1;
        // The code won't be TOTP-valid this far out, so just verify no stale rejection.
        // Generate a fresh code for the new timestamp.
        let fresh_code = validator.code_for_timestamp(after_expiry);
        assert!(validator.validate_at(&fresh_code, after_expiry).unwrap());
    }

    #[test]
    fn secret_is_generated_and_reused() {
        let dir = tempdir().unwrap();
        let store = SecretStore::new(dir.path(), true);

        let (first, first_uri) =
            OtpValidator::from_config(&test_config(), dir.path(), &store).unwrap();
        assert!(first_uri.is_some());

        let secret_path = secret_file_path(dir.path());
        let stored = fs::read_to_string(&secret_path).unwrap();
        assert!(SecretStore::is_encrypted(stored.trim()));

        let (second, second_uri) =
            OtpValidator::from_config(&test_config(), dir.path(), &store).unwrap();
        assert!(second_uri.is_none());

        let ts = 1_700_000_000u64;
        assert_eq!(first.code_for_timestamp(ts), second.code_for_timestamp(ts));
    }
}
