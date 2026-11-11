//! Shared proleptic Gregorian UTC date arithmetic; no locale or system-clock dependencies.

/// Gregorian date for signed days from 1970-01-01. Intermediate arithmetic is wide enough
/// for every i64 day value, including dates well beyond Unix timestamp ranges.
pub fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = i128::from(days) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    // At most ~2.6e16 years for any i64 day, which fits in i64.
    ((yoe + era * 400 + i128::from(month <= 2)) as i64, month, day)
}
pub fn utc_parts(seconds: i64) -> (i64, u32, u32, u32, u32, u32) {
    let (y, m, d) = civil_from_days(seconds.div_euclid(86_400));
    let s = seconds.rem_euclid(86_400);
    (y, m, d, (s / 3600) as u32, (s / 60 % 60) as u32, (s % 60) as u32)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn epoch_leap_and_negative_dates() {
        assert_eq!(utc_parts(0), (1970, 1, 1, 0, 0, 0));
        assert_eq!(utc_parts(-1), (1969, 12, 31, 23, 59, 59));
        assert_eq!(utc_parts(951_782_400), (2000, 2, 29, 0, 0, 0));
        for days in [i64::MIN, i64::MAX] {
            let (_, m, d) = civil_from_days(days);
            assert!((1..=12).contains(&m));
            assert!((1..=31).contains(&d));
        }
    }
}
