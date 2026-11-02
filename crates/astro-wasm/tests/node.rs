//! Runs in Node.js: `wasm-pack test --node crates/astro-wasm`.
#![cfg(target_arch = "wasm32")]

use astro_core::{
    CanChi, ConnectionCheck, ENGINE_VERSION, FourPillars, LunarDate, LunarYear, SolarDate,
    SolarDateTime,
};
use astro_wasm::{
    can_chi_name, check_connection, day_info, four_pillars, lunar_to_solar, lunar_year,
    solar_term_name, solar_terms_in_year, solar_to_lunar, true_solar_time,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::wasm_bindgen_test;

fn js<T: Serialize>(value: &T) -> JsValue {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .unwrap()
}

fn rust<T: DeserializeOwned>(result: Result<JsValue, JsValue>) -> T {
    let value = result.expect("call succeeds");
    serde_wasm_bindgen::from_value(value).expect("result has the declared shape")
}

const TET_2024: SolarDate = SolarDate {
    year: 2024,
    month: 2,
    day: 10,
};

#[wasm_bindgen_test]
fn connection_check_round_trips_a_date() {
    let check: ConnectionCheck = rust(check_connection(js(&TET_2024)));
    assert_eq!(check.date, TET_2024);
    assert_eq!(check.julian_day_number, 2_460_351);
    assert_eq!(check.engine_version, ENGINE_VERSION);
    assert!(
        check_connection(js(&SolarDate {
            day: 30,
            ..TET_2024
        }))
        .is_err()
    );
}

#[wasm_bindgen_test]
fn converts_between_calendars() {
    let lunar: LunarDate = rust(solar_to_lunar(js(&TET_2024), None));
    assert_eq!(
        lunar,
        LunarDate {
            year: 2024,
            month: 1,
            day: 1,
            is_leap_month: false
        }
    );
    let solar: SolarDate = rust(lunar_to_solar(js(&lunar), Some(7.0)));
    assert_eq!(solar, TET_2024);
}

#[wasm_bindgen_test]
fn time_zone_defaults_to_vietnam() {
    // Tết 2007 was 17 February in Vietnam and 18 February in China.
    let tet_2007 = js(&LunarDate {
        year: 2007,
        month: 1,
        day: 1,
        is_leap_month: false,
    });
    let vietnam: SolarDate = rust(lunar_to_solar(tet_2007.clone(), None));
    let china: SolarDate = rust(lunar_to_solar(tet_2007, Some(8.0)));
    assert_eq!((vietnam.day, china.day), (17, 18));
}

#[wasm_bindgen_test]
fn year_terms_and_day_info() {
    let year: LunarYear = rust(lunar_year(2025, None));
    assert_eq!(year.leap_month, Some(6));
    let terms: Vec<astro_core::SolarTermEvent> = rust(solar_terms_in_year(2024, None));
    assert_eq!(terms.len(), 24);
    let info: astro_core::DayInfo = rust(day_info(js(&TET_2024), None));
    assert_eq!(can_chi_name(js(&info.day_can_chi)).unwrap(), "Giáp Thìn");
    assert_eq!(solar_term_name(js(&info.solar_term)).unwrap(), "Lập Xuân");
}

#[wasm_bindgen_test]
fn four_pillars_fills_in_default_options() {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Options {
        rat_hour: &'static str,
    }
    let birth = SolarDateTime {
        year: 2024,
        month: 2,
        day: 10,
        hour: 23,
        minute: 30,
        second: 0,
    };
    let chart: FourPillars = rust(four_pillars(
        js(&birth),
        js(&Options {
            rat_hour: "SplitAtMidnight",
        }),
    ));
    assert_eq!(chart.day, CanChi::from_cycle_index(40));
    let solar: SolarDateTime = rust(true_solar_time(js(&birth), 105.0, None));
    assert_eq!(solar.hour, 23);
}

#[wasm_bindgen_test]
fn rejects_invalid_input() {
    assert!(
        solar_to_lunar(
            js(&SolarDate {
                year: 2023,
                month: 2,
                day: 29
            }),
            None
        )
        .is_err()
    );
    assert!(solar_to_lunar(JsValue::from_str("2024-02-10"), None).is_err());
    assert!(lunar_year(2024, Some(99.0)).is_err());
}
