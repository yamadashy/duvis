//! Standalone value parsers for filter inputs. Public so the CLI layer
//! can use them directly without constructing a [`super::Filter`] just
//! to validate one flag's value.

use anyhow::{anyhow, Result};

/// Parse a human-friendly byte count: bare integer = bytes, suffix
/// `B / K / KB / KiB / M / MB / MiB / G / GB / GiB / T / TB / TiB`
/// (case-insensitive). All multipliers are 1024-based to match
/// `format_size` and `du -h`. Floating-point is accepted: `1.5G`.
pub(crate) fn parse_size(s: &str) -> Result<u64> {
    let s = s.trim();
    if s.is_empty() {
        return Err(anyhow!("empty size"));
    }

    // Split numeric prefix from suffix. We accept floats so we can't
    // just take the longest digit run.
    let split = s
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(s.len());
    let (num_str, unit) = s.split_at(split);
    let num: f64 = num_str
        .parse()
        .map_err(|_| anyhow!("not a number: '{num_str}' in '{s}'"))?;
    if !num.is_finite() {
        return Err(anyhow!("not a finite number: '{s}'"));
    }
    if num.is_sign_negative() {
        return Err(anyhow!("negative size: '{s}'"));
    }

    let mult = match unit.trim().to_ascii_uppercase().as_str() {
        "" | "B" => 1u64,
        "K" | "KB" | "KIB" => 1024,
        "M" | "MB" | "MIB" => 1024 * 1024,
        "G" | "GB" | "GIB" => 1024 * 1024 * 1024,
        "T" | "TB" | "TIB" => 1024u64.pow(4),
        other => return Err(anyhow!("unknown size unit: '{other}' in '{s}'")),
    };

    // Range check before the cast: `f64 as u64` saturates silently to
    // u64::MAX on overflow, which would let `--min-size 99999P` quietly
    // become an effectively unmatchable filter instead of an error.
    // Use `>=` because `u64::MAX as f64` rounds *up* to 2^64 (u64::MAX
    // itself isn't representable in f64), so `> u64::MAX as f64` would
    // miss values that round-trip to exactly 2^64 and then saturate.
    let bytes = (num * mult as f64).round();
    if bytes >= u64::MAX as f64 {
        return Err(anyhow!("size out of range (exceeds u64): '{s}'"));
    }
    Ok(bytes as u64)
}

/// Parse a relative duration: `<integer><suffix>` where suffix is
/// `d` (days, also default), `w` (7d), `m` (30d), `y` (365d).
/// Returns the duration in *days*. Calendar-correctness is not the
/// goal here — agents asking "files older than 30 days" don't want
/// a date library, they want a fast cutoff.
pub(crate) fn parse_duration_days(s: &str) -> Result<u64> {
    let s = s.trim();
    if s.is_empty() {
        return Err(anyhow!("empty duration"));
    }

    let split = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    let (num_str, unit) = s.split_at(split);
    let num: u64 = num_str
        .parse()
        .map_err(|_| anyhow!("not an integer: '{num_str}' in '{s}'"))?;

    let days_per_unit = match unit.trim().to_ascii_lowercase().as_str() {
        "" | "d" => 1,
        "w" => 7,
        "m" => 30,
        "y" => 365,
        other => {
            return Err(anyhow!(
                "unknown duration unit: '{other}' in '{s}' (use d/w/m/y)"
            ))
        }
    };

    num.checked_mul(days_per_unit)
        .ok_or_else(|| anyhow!("duration overflow: '{s}'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_size_1024_based() {
        assert_eq!(parse_size("0").unwrap(), 0);
        assert_eq!(parse_size("100").unwrap(), 100);
        assert_eq!(parse_size("1K").unwrap(), 1024);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1KiB").unwrap(), 1024);
        assert_eq!(parse_size("1M").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1.5M").unwrap(), (1.5 * 1024.0 * 1024.0) as u64);
        assert_eq!(parse_size("1g").unwrap(), 1024u64.pow(3));
        assert_eq!(parse_size("1T").unwrap(), 1024u64.pow(4));
    }

    #[test]
    fn parse_size_rejects_garbage() {
        assert!(parse_size("").is_err());
        assert!(parse_size("xyz").is_err());
        assert!(parse_size("1XB").is_err());
        assert!(parse_size("-1M").is_err());
    }

    #[test]
    fn parse_size_rejects_non_finite_and_overflow() {
        // f64::parse accepts "inf"/"nan" — we must reject them, not let
        // them silently cast to u64::MAX / 0.
        assert!(parse_size("inf").is_err());
        assert!(parse_size("nan").is_err());
        // 1e20 bytes overflows u64 (max ≈ 1.8e19).
        assert!(parse_size("1e20").is_err());
        // Same magnitude with a unit multiplier.
        assert!(parse_size("99999999999T").is_err());
        // Boundary: u64::MAX is not representable in f64 (rounds up to 2^64),
        // so the literal value must also be rejected — otherwise it would
        // silently saturate via `as u64`.
        assert!(parse_size("18446744073709551615").is_err());
    }

    #[test]
    fn parse_duration_days_works() {
        assert_eq!(parse_duration_days("7").unwrap(), 7);
        assert_eq!(parse_duration_days("7d").unwrap(), 7);
        assert_eq!(parse_duration_days("2w").unwrap(), 14);
        assert_eq!(parse_duration_days("3m").unwrap(), 90);
        assert_eq!(parse_duration_days("1y").unwrap(), 365);
        assert_eq!(parse_duration_days("1Y").unwrap(), 365);
    }

    #[test]
    fn parse_duration_rejects_unknown_suffix() {
        assert!(parse_duration_days("7h").is_err());
        assert!(parse_duration_days("").is_err());
        assert!(parse_duration_days("abc").is_err());
    }
}
