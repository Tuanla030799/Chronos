//! ΔT = TT − UT: the difference between uniform Terrestrial Time, which the ephemerides use,
//! and Universal Time, which follows the Earth's irregular rotation.
//!
//! Polynomial expressions of Espenak & Meeus (2006), *Five Millennium Canon of Solar
//! Eclipses*, as published on NASA's eclipse web site. They follow observations to about a
//! second through the 20th century; later values are extrapolated (about 5 s too large by
//! 2025), which moves computed instants by seconds only.

use crate::math::polynomial;
use crate::time::{J2000, SECONDS_PER_DAY};

/// ΔT in seconds at `decimal_year` (e.g. 2024.5 for mid-2024).
pub fn delta_t_seconds(decimal_year: f64) -> f64 {
    let y = decimal_year;
    let long_term_parabola = || {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    };
    if y < -500.0 {
        long_term_parabola()
    } else if y < 500.0 {
        polynomial(
            y / 100.0,
            &[
                10583.6,
                -1014.41,
                33.78311,
                -5.952053,
                -0.1798452,
                0.022174192,
                0.0090316521,
            ],
        )
    } else if y < 1600.0 {
        polynomial(
            (y - 1000.0) / 100.0,
            &[
                1574.2,
                -556.01,
                71.23472,
                0.319781,
                -0.8503463,
                -0.005050998,
                0.0083572073,
            ],
        )
    } else if y < 1700.0 {
        polynomial(y - 1600.0, &[120.0, -0.9808, -0.01532, 1.0 / 7129.0])
    } else if y < 1800.0 {
        polynomial(
            y - 1700.0,
            &[8.83, 0.1603, -0.0059285, 0.00013336, -1.0 / 1_174_000.0],
        )
    } else if y < 1860.0 {
        polynomial(
            y - 1800.0,
            &[
                13.72,
                -0.332447,
                0.0068612,
                0.0041116,
                -0.00037436,
                0.0000121272,
                -0.0000001699,
                0.000000000875,
            ],
        )
    } else if y < 1900.0 {
        polynomial(
            y - 1860.0,
            &[
                7.62,
                0.5737,
                -0.251754,
                0.01680668,
                -0.0004473624,
                1.0 / 233_174.0,
            ],
        )
    } else if y < 1920.0 {
        polynomial(
            y - 1900.0,
            &[-2.79, 1.494119, -0.0598939, 0.0061966, -0.000197],
        )
    } else if y < 1941.0 {
        polynomial(y - 1920.0, &[21.20, 0.84493, -0.076100, 0.0020936])
    } else if y < 1961.0 {
        polynomial(y - 1950.0, &[29.07, 0.407, -1.0 / 233.0, 1.0 / 2547.0])
    } else if y < 1986.0 {
        polynomial(y - 1975.0, &[45.45, 1.067, -1.0 / 260.0, -1.0 / 718.0])
    } else if y < 2005.0 {
        polynomial(
            y - 2000.0,
            &[
                63.86,
                0.3345,
                -0.060374,
                0.0017275,
                0.000651814,
                0.00002373599,
            ],
        )
    } else if y < 2050.0 {
        polynomial(y - 2000.0, &[62.92, 0.32217, 0.005589])
    } else if y < 2150.0 {
        long_term_parabola() - 0.5628 * (2150.0 - y)
    } else {
        long_term_parabola()
    }
}

/// Decimal year of the instant `jd`, precise enough for ΔT.
fn decimal_year(jd: f64) -> f64 {
    2000.0 + (jd - J2000) / 365.2425
}

/// Converts a Julian Day on the UT scale to a Julian Ephemeris Day (TT).
pub fn jd_ut_to_jde(jd_ut: f64) -> f64 {
    jd_ut + delta_t_seconds(decimal_year(jd_ut)) / SECONDS_PER_DAY
}

/// Converts a Julian Ephemeris Day (TT) to a Julian Day on the UT scale.
pub fn jde_to_jd_ut(jde: f64) -> f64 {
    // ΔT is tabulated against UT, so evaluate it again at the first UT estimate; this makes
    // the conversion the inverse of `jd_ut_to_jde` even where ΔT changes quickly.
    let estimate = jde - delta_t_seconds(decimal_year(jde)) / SECONDS_PER_DAY;
    jde - delta_t_seconds(decimal_year(estimate)) / SECONDS_PER_DAY
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::assert_close;

    #[test]
    fn reproduces_the_published_coefficients_at_segment_epochs() {
        assert_close(delta_t_seconds(1700.0), 8.83, 1e-9);
        assert_close(delta_t_seconds(1800.0), 13.72, 1e-9);
        assert_close(delta_t_seconds(1900.0), -2.79, 1e-9);
        assert_close(delta_t_seconds(1950.0), 29.07, 1e-9);
        assert_close(delta_t_seconds(1975.0), 45.45, 1e-9);
        assert_close(delta_t_seconds(2000.0), 63.86, 1e-9);
    }

    #[test]
    fn follows_observed_values() {
        // Observed ΔT (IERS) at the start of each year, in seconds.
        for (year, observed, tolerance) in [
            (1960.0, 33.15, 0.5),
            (1980.0, 50.54, 0.5),
            (2000.0, 63.83, 0.5),
            (2010.0, 66.07, 1.0),
            (2020.0, 69.36, 3.0),
        ] {
            assert_close(delta_t_seconds(year), observed, tolerance);
        }
    }

    #[test]
    fn segments_join_without_jumps() {
        for boundary in [
            1600.0, 1700.0, 1800.0, 1860.0, 1900.0, 1920.0, 1941.0, 1961.0, 1986.0, 2005.0, 2050.0,
            2150.0,
        ] {
            let jump = delta_t_seconds(boundary) - delta_t_seconds(boundary - 1e-9);
            assert!(jump.abs() < 0.3, "ΔT jumps by {jump} s at {boundary}");
        }
    }

    #[test]
    fn ut_and_tt_conversions_are_inverse() {
        for jd in [2_305_448.0, 2_451_545.0, 2_460_351.3, 2_597_641.0] {
            assert_close(jde_to_jd_ut(jd_ut_to_jde(jd)), jd, 1e-9);
        }
        // TT runs about 64 s ahead of UT in 2000.
        assert_close((jd_ut_to_jde(J2000) - J2000) * SECONDS_PER_DAY, 63.86, 0.01);
    }
}
