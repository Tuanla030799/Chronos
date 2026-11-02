//! Apparent position of the Sun and the equation of time.
//!
//! The geometric position comes from the truncated VSOP87 theory of Meeus' appendix III
//! (about 1″), reduced to the FK5 frame and corrected for nutation and aberration (Meeus
//! ch. 25). Nutation uses the abridged series of Meeus ch. 22 (0.5″ in longitude, 0.1″ in
//! obliquity). One arcsecond of solar longitude is about 24 seconds of time.

use crate::astronomy::vsop87_earth::{self, Term};
use crate::math::{angle_difference, normalize_degrees, polynomial};
use crate::time::J2000;

/// Degrees in one second of arc.
const ARCSECOND: f64 = 1.0 / 3600.0;

/// Apparent geocentric position of the Sun, with the quantities used to derive it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ApparentSun {
    /// Apparent ecliptic longitude λ in degrees, [0, 360), referred to the true equinox of date.
    pub longitude: f64,
    /// Ecliptic latitude β in degrees.
    pub latitude: f64,
    /// Earth–Sun distance in astronomical units.
    pub distance: f64,
    /// Nutation in longitude Δψ in degrees.
    pub nutation_in_longitude: f64,
    /// True obliquity of the ecliptic ε in degrees.
    pub true_obliquity: f64,
}

/// Apparent position of the Sun at `jde` (Julian Ephemeris Day, TT).
pub fn apparent_sun(jde: f64) -> ApparentSun {
    let (l, b, r) = earth_heliocentric(jde);
    let t = (jde - J2000) / 36_525.0;
    // The Sun seen from the Earth lies opposite the Earth seen from the Sun.
    let theta = normalize_degrees(l.to_degrees() + 180.0);
    let beta = -b.to_degrees();
    // VSOP87 dynamical frame to FK5 (Meeus eq. 32.3).
    let lambda_prime = (theta - 1.397 * t - 0.000_31 * t * t).to_radians();
    let theta_fk5 = theta - 0.090_33 * ARCSECOND;
    let beta_fk5 = beta + 0.039_16 * (lambda_prime.cos() - lambda_prime.sin()) * ARCSECOND;
    let (nutation_in_longitude, nutation_in_obliquity) = nutation(jde);
    let aberration = -20.489_8 * ARCSECOND / r;
    ApparentSun {
        longitude: normalize_degrees(theta_fk5 + nutation_in_longitude + aberration),
        latitude: beta_fk5,
        distance: r,
        nutation_in_longitude,
        true_obliquity: mean_obliquity(jde) + nutation_in_obliquity,
    }
}

/// Apparent geocentric ecliptic longitude of the Sun in degrees, [0, 360), at `jde` (TT).
pub fn apparent_longitude(jde: f64) -> f64 {
    apparent_sun(jde).longitude
}

/// Nutation in longitude Δψ and in obliquity Δε, in degrees, at `jde` (TT).
pub fn nutation(jde: f64) -> (f64, f64) {
    let t = (jde - J2000) / 36_525.0;
    let moon_node = polynomial(
        t,
        &[125.044_52, -1_934.136_261, 0.002_070_8, 1.0 / 450_000.0],
    )
    .to_radians();
    let sun_mean_longitude = (280.466_5 + 36_000.769_8 * t).to_radians();
    let moon_mean_longitude = (218.316_5 + 481_267.881_3 * t).to_radians();
    let delta_psi = -17.20 * moon_node.sin()
        - 1.32 * (2.0 * sun_mean_longitude).sin()
        - 0.23 * (2.0 * moon_mean_longitude).sin()
        + 0.21 * (2.0 * moon_node).sin();
    let delta_epsilon = 9.20 * moon_node.cos()
        + 0.57 * (2.0 * sun_mean_longitude).cos()
        + 0.10 * (2.0 * moon_mean_longitude).cos()
        - 0.09 * (2.0 * moon_node).cos();
    (delta_psi * ARCSECOND, delta_epsilon * ARCSECOND)
}

/// Mean obliquity of the ecliptic ε₀ in degrees at `jde` (TT), Meeus eq. 22.2.
pub fn mean_obliquity(jde: f64) -> f64 {
    let t = (jde - J2000) / 36_525.0;
    polynomial(t, &[84_381.448, -46.815_0, -0.000_59, 0.001_813]) * ARCSECOND
}

/// Equation of time in minutes at `jde` (TT): apparent minus mean solar time (Meeus eq. 28.1).
///
/// Positive when the Sun runs ahead of the clock (about +16.4 min in early November),
/// negative when it lags (about −14.2 min in mid-February).
pub fn equation_of_time(jde: f64) -> f64 {
    let tau = (jde - J2000) / 365_250.0;
    let sun_mean_longitude = polynomial(
        tau,
        &[
            280.466_456_7,
            360_007.698_277_9,
            0.030_320_28,
            1.0 / 49_931.0,
            -1.0 / 15_300.0,
            -1.0 / 2_000_000.0,
        ],
    );
    let sun = apparent_sun(jde);
    let lambda = sun.longitude.to_radians();
    let beta = sun.latitude.to_radians();
    let epsilon = sun.true_obliquity.to_radians();
    let right_ascension = (lambda.sin() * epsilon.cos() - beta.tan() * epsilon.sin())
        .atan2(lambda.cos())
        .to_degrees();
    let degrees = sun_mean_longitude - 0.005_718_3 - right_ascension
        + sun.nutation_in_longitude * epsilon.cos();
    // Four minutes of time per degree.
    angle_difference(degrees, 0.0) * 4.0
}

/// Heliocentric ecliptic longitude (rad), latitude (rad) and radius vector (AU) of the Earth.
fn earth_heliocentric(jde: f64) -> (f64, f64, f64) {
    let tau = (jde - J2000) / 365_250.0;
    (
        vsop87(&vsop87_earth::L, tau),
        vsop87(&vsop87_earth::B, tau),
        vsop87(&vsop87_earth::R, tau),
    )
}

/// Sums one VSOP87 coordinate, Σₙ τⁿ · Σ A·cos(B + C·τ), scaled from 10⁻⁸ units.
fn vsop87(series: &[&[Term]], tau: f64) -> f64 {
    let sum = series.iter().rev().fold(0.0, |higher_orders, terms| {
        higher_orders * tau
            + terms
                .iter()
                .map(|&(a, b, c)| a * (b + c * tau).cos())
                .sum::<f64>()
    });
    sum * 1e-8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::delta_t::jd_ut_to_jde;
    use crate::test_support::{assert_close, utc};
    use core::f64::consts::TAU;

    /// 1992 October 13.0 TD, the date of Meeus' worked examples 25.b and 28.a.
    const MEEUS_1992_OCT_13: f64 = 2_448_908.5;

    #[test]
    fn vsop87_matches_meeus_example_25b() {
        let (l, b, r) = earth_heliocentric(MEEUS_1992_OCT_13);
        assert_close(
            l.rem_euclid(TAU),
            (-43.634_847_96_f64).rem_euclid(TAU),
            1e-8,
        );
        assert_close(b, -0.000_003_12, 1e-8);
        assert_close(r, 0.997_607_75, 1e-8);
    }

    #[test]
    fn apparent_longitude_matches_meeus_example_25b() {
        // λ = 199°54′21.818″ with the full nutation series; the abridged one is good to 0.5″.
        let expected = 199.0 + 54.0 / 60.0 + 21.818 / 3600.0;
        assert_close(
            apparent_longitude(MEEUS_1992_OCT_13),
            expected,
            0.5 * ARCSECOND,
        );
    }

    #[test]
    fn nutation_matches_meeus_example_22a() {
        // 1987 April 10, 0h TD: Δψ = −3.788″, Δε = +9.443″, ε₀ = 23°26′27.407″.
        let jde = 2_446_895.5;
        let (delta_psi, delta_epsilon) = nutation(jde);
        assert_close(delta_psi / ARCSECOND, -3.788, 0.5);
        assert_close(delta_epsilon / ARCSECOND, 9.443, 0.1);
        let expected_obliquity = 23.0 + 26.0 / 60.0 + 27.407 / 3600.0;
        assert_close(mean_obliquity(jde), expected_obliquity, 0.001 * ARCSECOND);
    }

    #[test]
    fn equation_of_time_matches_meeus_example_28a() {
        // E = +3.427351° = +13.70940 min.
        assert_close(equation_of_time(MEEUS_1992_OCT_13), 13.709_40, 0.01);
    }

    #[test]
    fn equation_of_time_reaches_its_annual_extremes() {
        let november = equation_of_time(jd_ut_to_jde(utc(2024, 11, 3, 12, 0)));
        let february = equation_of_time(jd_ut_to_jde(utc(2024, 2, 11, 12, 0)));
        assert!(
            (16.0..16.7).contains(&november),
            "early November: {november} min"
        );
        assert!(
            (-14.5..-13.9).contains(&february),
            "mid-February: {february} min"
        );
    }

    #[test]
    fn longitude_stays_in_one_turn_and_advances_about_a_degree_a_day() {
        let start = jd_ut_to_jde(utc(2024, 3, 19, 0, 0));
        for day in 0..3 {
            let today = apparent_longitude(start + f64::from(day));
            let tomorrow = apparent_longitude(start + f64::from(day) + 1.0);
            assert!((0.0..360.0).contains(&today));
            assert_close(angle_difference(tomorrow, today), 0.99, 0.03);
        }
    }
}
