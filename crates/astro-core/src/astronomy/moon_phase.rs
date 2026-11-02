//! Instants of the true new moon (Sóc), Meeus *Astronomical Algorithms* ch. 49.
//!
//! The mean phase is corrected with the 25 periodic terms for the new moon and the 14
//! planetary arguments, which Meeus reports as accurate to a few seconds for modern dates.
//!
//! New moons are numbered from the one of 1 January 1900 (k = 0), as in Hồ Ngọc Đức's
//! algorithm; Meeus numbers them from 6 January 2000, which is k = 1237 here.

use crate::astronomy::delta_t::jde_to_jd_ut;
use crate::time::local_jdn;

/// Mean length of the synodic month, in days.
pub const SYNODIC_MONTH: f64 = 29.530_588_861;

/// Number of the new moon of 2000-01-06, the zero of Meeus' numbering.
const MEEUS_ZERO_LUNATION: i32 = 1237;

/// JDE of the mean new moon of 2000-01-06.
const MEEUS_ZERO_JDE: f64 = 2_451_550.097_66;

/// Planetary arguments A2–A14: (degrees at k = 0, degrees per lunation, amplitude in days).
const PLANETARY_ARGUMENTS: [(f64, f64, f64); 13] = [
    (251.88, 0.016_321, 0.000_165),
    (251.83, 26.651_886, 0.000_164),
    (349.42, 36.412_478, 0.000_126),
    (84.66, 18.206_239, 0.000_110),
    (141.74, 53.303_771, 0.000_062),
    (207.14, 2.453_732, 0.000_060),
    (154.84, 7.306_860, 0.000_056),
    (34.52, 27.261_239, 0.000_047),
    (207.19, 0.121_824, 0.000_042),
    (291.34, 1.844_379, 0.000_040),
    (161.72, 24.198_154, 0.000_037),
    (239.56, 25.513_099, 0.000_035),
    (331.55, 3.592_518, 0.000_023),
];

/// Julian Ephemeris Day (TT) of the true new moon number `k`, counted from the new moon of
/// 1900-01-01 (`k = 0`); `k` grows by one each synodic month.
pub fn new_moon_jde(k: i32) -> f64 {
    // Meeus' formulas use his own numbering, with k = 0 on 2000-01-06.
    let k = f64::from(k - MEEUS_ZERO_LUNATION);
    let t = k / 1236.85;
    let (t2, t3, t4) = (t * t, t * t * t, t * t * t * t);
    let mean = MEEUS_ZERO_JDE + SYNODIC_MONTH * k + 0.000_154_37 * t2 - 0.000_000_150 * t3
        + 0.000_000_000_73 * t4;

    // Eccentricity factor of the Earth's orbit and the fundamental arguments.
    let e = 1.0 - 0.002_516 * t - 0.000_007_4 * t2;
    let m = (2.553_4 + 29.105_356_70 * k - 0.000_001_4 * t2 - 0.000_000_11 * t3).to_radians();
    let mp = (201.564_3 + 385.816_935_28 * k + 0.010_758_2 * t2 + 0.000_012_38 * t3
        - 0.000_000_058 * t4)
        .to_radians();
    let f = (160.710_8 + 390.670_502_84 * k - 0.001_611_8 * t2 - 0.000_002_27 * t3
        + 0.000_000_011 * t4)
        .to_radians();
    let omega = (124.774_6 - 1.563_755_88 * k + 0.002_067_2 * t2 + 0.000_002_15 * t3).to_radians();

    let periodic = -0.407_20 * mp.sin()
        + 0.172_41 * e * m.sin()
        + 0.016_08 * (2.0 * mp).sin()
        + 0.010_39 * (2.0 * f).sin()
        + 0.007_39 * e * (mp - m).sin()
        - 0.005_14 * e * (mp + m).sin()
        + 0.002_08 * e * e * (2.0 * m).sin()
        - 0.001_11 * (mp - 2.0 * f).sin()
        - 0.000_57 * (mp + 2.0 * f).sin()
        + 0.000_56 * e * (2.0 * mp + m).sin()
        - 0.000_42 * (3.0 * mp).sin()
        + 0.000_42 * e * (m + 2.0 * f).sin()
        + 0.000_38 * e * (m - 2.0 * f).sin()
        - 0.000_24 * e * (2.0 * mp - m).sin()
        - 0.000_17 * omega.sin()
        - 0.000_07 * (mp + 2.0 * m).sin()
        + 0.000_04 * (2.0 * mp - 2.0 * f).sin()
        + 0.000_04 * (3.0 * m).sin()
        + 0.000_03 * (mp + m - 2.0 * f).sin()
        + 0.000_03 * (2.0 * mp + 2.0 * f).sin()
        - 0.000_03 * (mp + m + 2.0 * f).sin()
        + 0.000_03 * (mp - m + 2.0 * f).sin()
        - 0.000_02 * (mp - m - 2.0 * f).sin()
        - 0.000_02 * (3.0 * mp + m).sin()
        + 0.000_02 * (4.0 * mp).sin();

    let a1 = (299.77 + 0.107_408 * k - 0.009_173 * t2).to_radians();
    let planetary = 0.000_325 * a1.sin()
        + PLANETARY_ARGUMENTS
            .iter()
            .map(|&(base, rate, amplitude)| amplitude * (base + rate * k).to_radians().sin())
            .sum::<f64>();

    mean + periodic + planetary
}

/// Julian Day (UT) of the true new moon number `k`.
pub fn new_moon_jd_ut(k: i32) -> f64 {
    jde_to_jd_ut(new_moon_jde(k))
}

/// Local date, as a Julian Day Number, of new moon `k` in a zone `tz_offset` hours east of
/// UTC: the first day (mùng 1) of a lunar month.
pub fn new_moon_day(k: i32, tz_offset: f64) -> i32 {
    local_jdn(new_moon_jd_ut(k), tz_offset)
}

/// Number of the last new moon whose local date is on or before day `jdn`.
pub fn lunation_on_or_before(jdn: i32, tz_offset: f64) -> i32 {
    let mut k =
        ((f64::from(jdn) - MEEUS_ZERO_JDE) / SYNODIC_MONTH).floor() as i32 + MEEUS_ZERO_LUNATION;
    while new_moon_day(k, tz_offset) > jdn {
        k -= 1;
    }
    while new_moon_day(k + 1, tz_offset) <= jdn {
        k += 1;
    }
    k
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{CN, VN, assert_close, date, utc};

    fn lunation_nearest(jd: f64) -> i32 {
        ((jd - MEEUS_ZERO_JDE) / SYNODIC_MONTH).round() as i32 + MEEUS_ZERO_LUNATION
    }

    #[test]
    fn matches_meeus_example_49a() {
        // New moon of 1977 February: Meeus' k = −283, i.e. 954 from 1900; JDE 2443192.65118
        // (18 Feb, 3h37m42s TD).
        assert_close(new_moon_jde(954), 2_443_192.651_18, 1e-5);
    }

    #[test]
    fn numbering_starts_with_the_new_moon_of_1_january_1900() {
        // 1900-01-01 13:51 UTC and 2000-01-06 18:14 UTC.
        assert_close(new_moon_jd_ut(0), utc(1900, 1, 1, 13, 51), 1.5 / 1440.0);
        assert_close(new_moon_jd_ut(1237), utc(2000, 1, 6, 18, 14), 1.5 / 1440.0);
    }

    #[test]
    fn matches_published_new_moon_times() {
        // UTC times to the minute from the USNO / NASA lunar phase tables.
        for published in [
            utc(1968, 1, 29, 16, 29),
            utc(2000, 1, 6, 18, 14),
            utc(2007, 2, 17, 16, 14),
            utc(2017, 8, 21, 18, 30),
            utc(2023, 3, 21, 17, 23),
            utc(2023, 4, 20, 4, 12),
            utc(2024, 4, 8, 18, 21),
            utc(2025, 1, 29, 12, 36),
        ] {
            let computed = new_moon_jd_ut(lunation_nearest(published));
            assert_close(computed, published, 1.5 / 1440.0);
        }
    }

    #[test]
    fn new_moon_day_depends_on_the_time_zone() {
        // 16:14 UTC on 17 February 2007 is 23:14 in Hà Nội but 00:14 on the 18th in Beijing.
        let k = lunation_nearest(utc(2007, 2, 17, 16, 14));
        assert_eq!(new_moon_day(k, VN), date(2007, 2, 17).to_jdn());
        assert_eq!(new_moon_day(k, CN), date(2007, 2, 18).to_jdn());
    }

    #[test]
    fn lunation_on_or_before_brackets_the_day() {
        let new_moon = date(2024, 2, 10).to_jdn();
        let k = lunation_on_or_before(new_moon, VN);
        assert_eq!(new_moon_day(k, VN), new_moon);
        assert_eq!(lunation_on_or_before(new_moon - 1, VN), k - 1);
        assert_eq!(lunation_on_or_before(new_moon + 28, VN), k);
        for jdn in date(1990, 1, 1).to_jdn()..date(1991, 1, 1).to_jdn() {
            let k = lunation_on_or_before(jdn, VN);
            assert!(new_moon_day(k, VN) <= jdn && jdn < new_moon_day(k + 1, VN));
        }
    }
}
