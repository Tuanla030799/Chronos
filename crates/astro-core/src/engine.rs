//! Engine identity and the connection check the bindings use to prove they reach the core.

use crate::error::AstroError;
use crate::time::SolarDate;

/// Version of this astro-core build, from its Cargo manifest.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Answer of [`check_connection`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ConnectionCheck {
    /// Version of the astro-core build that answered.
    pub engine_version: String,
    /// The date received from the caller, after validation.
    pub date: SolarDate,
    /// Julian Day Number of `date`, computed by the core.
    pub julian_day_number: i32,
}

/// Round-trips `date` through the core: validates it and returns it with its Julian Day Number
/// and the engine version. An invalid date returns [`AstroError::InvalidSolarDate`], so a
/// binding can exercise both its success and its error path.
pub fn check_connection(date: SolarDate) -> Result<ConnectionCheck, AstroError> {
    date.validate()?;
    Ok(ConnectionCheck {
        engine_version: ENGINE_VERSION.to_owned(),
        date,
        julian_day_number: date.to_jdn(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::date;

    #[test]
    fn echoes_a_valid_date_with_its_julian_day_number() {
        let check = check_connection(date(2024, 2, 10)).unwrap();
        assert_eq!(check.date, date(2024, 2, 10));
        assert_eq!(check.julian_day_number, 2_460_351);
        assert_eq!(check.engine_version, ENGINE_VERSION);
    }

    #[test]
    fn reports_a_date_that_does_not_exist() {
        assert_eq!(
            check_connection(SolarDate {
                year: 2023,
                month: 2,
                day: 29
            }),
            Err(AstroError::InvalidSolarDate {
                year: 2023,
                month: 2,
                day: 29
            })
        );
    }
}
