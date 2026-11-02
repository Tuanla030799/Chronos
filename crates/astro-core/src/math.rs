//! Small numeric helpers shared by the astronomical modules.

/// Reduces an angle to the range [0°, 360°).
pub(crate) fn normalize_degrees(degrees: f64) -> f64 {
    let reduced = degrees.rem_euclid(360.0);
    // `rem_euclid` rounds tiny negative inputs up to exactly 360.
    if reduced >= 360.0 { 0.0 } else { reduced }
}

/// Signed difference `to − from`, reduced to the range (−180°, 180°].
pub(crate) fn angle_difference(to: f64, from: f64) -> f64 {
    let difference = normalize_degrees(to - from);
    if difference > 180.0 {
        difference - 360.0
    } else {
        difference
    }
}

/// Evaluates a polynomial whose `coefficients` are in ascending powers of `x` (Horner's scheme).
pub(crate) fn polynomial(x: f64, coefficients: &[f64]) -> f64 {
    coefficients
        .iter()
        .rev()
        .fold(0.0, |sum, &coefficient| sum * x + coefficient)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_into_one_turn() {
        assert_eq!(normalize_degrees(-30.0), 330.0);
        assert_eq!(normalize_degrees(720.0), 0.0);
        assert_eq!(normalize_degrees(359.5), 359.5);
        assert_eq!(normalize_degrees(-1e-15), 0.0);
    }

    #[test]
    fn angle_difference_takes_the_short_way_round() {
        assert_eq!(angle_difference(10.0, 350.0), 20.0);
        assert_eq!(angle_difference(350.0, 10.0), -20.0);
        assert_eq!(angle_difference(180.0, 0.0), 180.0);
        assert_eq!(angle_difference(0.0, 180.0), 180.0);
    }

    #[test]
    fn evaluates_polynomials_in_ascending_powers() {
        assert_eq!(polynomial(2.0, &[1.0, -3.0, 0.5]), 1.0 - 6.0 + 2.0);
        assert_eq!(polynomial(5.0, &[]), 0.0);
    }
}
