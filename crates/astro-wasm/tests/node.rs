//! Runs in Node.js: `wasm-pack test --node crates/astro-wasm`.
#![cfg(target_arch = "wasm32")]

use astro_core::almanac::Luck;
use astro_core::fengshui::{
    CompassDirection, DirectionStar, EightMansionsStar, FlyingStar, KuaCompatibility, KuaInfo,
    LoShuGrid, NineStar, NineStars, Palace, TamNguyen, Trigram, TrigramGroup,
};
use astro_core::{
    CanChi, ConnectionCheck, ENGINE_VERSION, FiveElements, FourPillars, LunarDate, LunarYear,
    SolarDate, SolarDateTime,
};
use astro_wasm::{
    age_restrictions, almanac_day, analyze_bazi, branch_name, can_chi_name, check_connection,
    compass_direction_name, day_info, day_nine_star, direction_trigram, eight_mansions_chart,
    eight_mansions_star_corresponding_name, eight_mansions_star_han_name, eight_mansions_star_luck,
    eight_mansions_star_name, five_elements_name, four_pillars, generate_loshu_grid,
    hour_nine_star, house_trigram, house_trigram_from_facing, kim_lau_name, kua_compatibility,
    kua_info, lo_shu_rows, lo_shu_star_at, lunar_to_solar, lunar_year, main_hidden_stem,
    month_nine_star, nguyen_of_year, nine_star_color_name, nine_star_element, nine_star_han_name,
    nine_star_home_palace, nine_star_luck, nine_star_name, nine_star_number,
    nine_star_xuan_kong_name, nine_stars, opposite_direction, palace_direction, palace_element,
    palace_han_name, palace_name, palace_number, palace_trigram, solar_term_name,
    solar_terms_in_year, solar_to_lunar, star_at, star_of_year, star_towards, stem_name,
    tam_nguyen_name, trigram_direction, trigram_element, trigram_group, trigram_group_name,
    trigram_han_name, trigram_name, trigram_number, trigram_palace, truc_name, true_solar_time,
    van_of_year, year_nine_star,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen]
extern "C" {
    /// `JSON.stringify`: shows a result as JavaScript sees it, which `rust` cannot, because it
    /// reads the names back through the serde derives that wrote them.
    #[wasm_bindgen(js_namespace = JSON)]
    fn stringify(value: &JsValue) -> String;
}

fn js<T: Serialize>(value: &T) -> JsValue {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .unwrap()
}

fn rust<T: DeserializeOwned>(result: Result<JsValue, JsValue>) -> T {
    let value = result.expect("call succeeds");
    serde_wasm_bindgen::from_value(value).expect("result has the declared shape")
}

/// An enum as a JavaScript caller writes it: the name of the variant.
fn variant(name: &str) -> JsValue {
    JsValue::from_str(name)
}

fn numbers(rows: [[NineStar; 3]; 3]) -> [[u8; 3]; 3] {
    rows.map(|row| row.map(NineStar::number))
}

const TET_2024: SolarDate = SolarDate {
    year: 2024,
    month: 2,
    day: 10,
};

/// Vector 2 of `docs/FENGSHUI_SPEC.md`: giờ Tý Sớm, past Hạ Chí but before its Giáp Tý day.
const VECTOR_2: SolarDateTime = SolarDateTime {
    year: 2000,
    month: 6,
    day: 25,
    hour: 23,
    minute: 30,
    second: 0,
};

/// Vector 3 of `docs/FENGSHUI_SPEC.md`: after Tết, before Lập Xuân.
const VECTOR_3: SolarDateTime = SolarDateTime {
    year: 2022,
    month: 2,
    day: 2,
    hour: 8,
    minute: 0,
    second: 0,
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
fn analysis_almanac_and_age_restrictions() {
    let birth = SolarDateTime {
        year: 2024,
        month: 2,
        day: 10,
        hour: 8,
        minute: 0,
        second: 0,
    };
    let analysis: astro_core::BaziAnalysis = rust(analyze_bazi(
        js(&birth),
        js(&std::collections::BTreeMap::<&str, &str>::new()),
        JsValue::from_str("Male"),
    ));
    assert!(analysis.dai_van.forward);

    let main_stem = main_hidden_stem(JsValue::from_str("Dan")).unwrap();
    assert_eq!(stem_name(main_stem).unwrap(), "Giáp");
    assert_eq!(branch_name(JsValue::from_str("Ti")).unwrap(), "Tỵ");

    let day: astro_core::AlmanacDay = rust(almanac_day(js(&TET_2024), None));
    assert_eq!(truc_name(js(&day.truc)).unwrap(), "Mãn");

    let restrictions: astro_core::AgeRestrictions = rust(age_restrictions(1990, 2026));
    assert_eq!(restrictions.lunar_age, 37);
    assert_eq!(
        kim_lau_name(js(&restrictions.kim_lau.unwrap())).unwrap(),
        "Kim Lâu Thân"
    );
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

#[wasm_bindgen_test]
fn kua_info_and_compatibility_of_two_people() {
    let man: KuaInfo = rust(kua_info(1990, variant("Male")));
    assert_eq!(
        (man.trigram, man.raw_number, man.number, man.group),
        (Trigram::Kham, 1, 1, TrigramGroup::DongTu)
    );

    let woman: KuaInfo = rust(kua_info(1990, variant("Female")));
    assert_eq!(
        (woman.trigram, woman.raw_number, woman.number, woman.group),
        (Trigram::CanTho, 5, 8, TrigramGroup::TayTu)
    );

    let couple: KuaCompatibility = rust(kua_compatibility(js(&man), js(&woman)));
    assert_eq!(
        (couple.star, couple.luck),
        (EightMansionsStar::NguQuy, Luck::Hung)
    );
    assert_eq!((couple.first, couple.second), (man, woman));
}

#[wasm_bindgen_test]
fn house_trigram_is_read_from_the_sitting_side() {
    let sitting: Trigram = rust(house_trigram(variant("Bac")));
    let facing: Trigram = rust(house_trigram_from_facing(variant("Nam")));
    assert_eq!((sitting, facing), (Trigram::Kham, Trigram::Kham));

    let opposite: CompassDirection = rust(opposite_direction(variant("Nam")));
    assert_eq!(opposite, CompassDirection::Bac);
    let trigram: Trigram = rust(direction_trigram(variant("DongBac")));
    assert_eq!(trigram, Trigram::CanTho);
}

#[wasm_bindgen_test]
fn eight_mansions_chart_runs_clockwise_from_north() {
    use CompassDirection::{Bac, Dong, DongBac, DongNam, Nam, Tay, TayBac, TayNam};
    use EightMansionsStar::{DienNien, HoaHai, LucSat, NguQuy, PhucVi, SinhKhi, ThienY, TuyetMenh};

    let chart: Vec<DirectionStar> = rust(eight_mansions_chart(variant("Kham")));
    let expected = [
        (Bac, PhucVi),
        (DongBac, NguQuy),
        (Dong, ThienY),
        (DongNam, SinhKhi),
        (Nam, DienNien),
        (TayNam, TuyetMenh),
        (Tay, HoaHai),
        (TayBac, LucSat),
    ]
    .map(|(direction, star)| DirectionStar { direction, star });
    assert_eq!(chart, expected);

    let towards: EightMansionsStar = rust(star_towards(variant("Kham"), variant("CanTho")));
    let at: EightMansionsStar = rust(star_at(variant("Kham"), variant("DongBac")));
    assert_eq!((towards, at), (NguQuy, NguQuy));
    let towards: EightMansionsStar = rust(star_towards(variant("Kham"), variant("Ly")));
    let at: EightMansionsStar = rust(star_at(variant("Kham"), variant("Nam")));
    assert_eq!((towards, at), (DienNien, DienNien));
}

#[wasm_bindgen_test]
fn nine_stars_of_vector_2_with_the_defaults_omitted() {
    let stars: NineStars = rust(nine_stars(js(&VECTOR_2), None, JsValue::UNDEFINED));
    assert_eq!(
        stars,
        NineStars {
            nguyen: TamNguyen::HaNguyen,
            van: NineStar::ThatXich,
            year: NineStar::CuuTu,
            month: NineStar::NhatBach,
            day: FlyingStar {
                star: NineStar::CuuTu,
                forward: true
            },
            hour: FlyingStar {
                star: NineStar::TamBich,
                forward: false
            },
        }
    );

    let year: NineStar = rust(year_nine_star(js(&VECTOR_2), None, JsValue::UNDEFINED));
    let month: NineStar = rust(month_nine_star(js(&VECTOR_2), None, JsValue::UNDEFINED));
    let day: FlyingStar = rust(day_nine_star(js(&VECTOR_2.date()), None));
    let hour: FlyingStar = rust(hour_nine_star(js(&VECTOR_2), None));
    assert_eq!(
        (year, month, day, hour),
        (stars.year, stars.month, stars.day, stars.hour)
    );
}

#[wasm_bindgen_test]
fn year_numbers_give_star_period_and_era() {
    let star: NineStar = rust(star_of_year(2024));
    let van: NineStar = rust(van_of_year(2024));
    let nguyen: TamNguyen = rust(nguyen_of_year(2024));
    assert_eq!(
        (star, van, nguyen),
        (NineStar::TamBich, NineStar::CuuTu, TamNguyen::HaNguyen)
    );
}

#[wasm_bindgen_test]
fn boundary_defaults_to_the_day_of_lap_xuan() {
    let stars_with = |boundary: JsValue| -> NineStars {
        let year: NineStar = rust(year_nine_star(js(&VECTOR_3), None, boundary.clone()));
        let month: NineStar = rust(month_nine_star(js(&VECTOR_3), None, boundary.clone()));
        let stars: NineStars = rust(nine_stars(js(&VECTOR_3), None, boundary));
        assert_eq!((year, month), (stars.year, stars.month));
        stars
    };

    let by_default = stars_with(JsValue::UNDEFINED);
    assert_eq!(
        (by_default.year, by_default.month),
        (NineStar::LucBach, NineStar::TamBich)
    );
    assert_eq!(stars_with(JsValue::NULL), by_default);
    assert_eq!(stars_with(variant("LapXuanDay")), by_default);

    let by_tet = stars_with(variant("LunarNewYear"));
    assert_eq!(
        (by_tet.year, by_tet.month),
        (NineStar::NguHoang, NineStar::CuuTu)
    );

    // Lập Xuân 2022 begins at 03:50, so 00:30 is in the new year by day only.
    let before_the_instant = SolarDateTime {
        day: 4,
        hour: 0,
        minute: 30,
        ..VECTOR_3
    };
    let by_day: NineStar = rust(year_nine_star(
        js(&before_the_instant),
        None,
        JsValue::UNDEFINED,
    ));
    let by_instant: NineStar = rust(year_nine_star(
        js(&before_the_instant),
        None,
        variant("LapXuanExact"),
    ));
    assert_eq!(
        (by_day, by_instant),
        (NineStar::NguHoang, NineStar::LucBach)
    );
}

#[wasm_bindgen_test]
fn star_zone_defaults_to_vietnam() {
    // Lập Xuân 2009 began on 3 February in Vietnam and on 4 February in China.
    let lap_xuan_2009 = js(&SolarDateTime {
        year: 2009,
        month: 2,
        day: 3,
        hour: 12,
        minute: 0,
        second: 0,
    });
    let year_and_month = |tz_offset: Option<f64>| -> (NineStar, NineStar) {
        let time = || lap_xuan_2009.clone();
        let year: NineStar = rust(year_nine_star(time(), tz_offset, JsValue::UNDEFINED));
        let month: NineStar = rust(month_nine_star(time(), tz_offset, JsValue::UNDEFINED));
        let stars: NineStars = rust(nine_stars(time(), tz_offset, JsValue::UNDEFINED));
        assert_eq!((year, month), (stars.year, stars.month));
        (year, month)
    };
    assert_eq!(year_and_month(None), (NineStar::CuuTu, NineStar::NguHoang));
    assert_eq!(
        year_and_month(Some(8.0)),
        (NineStar::NhatBach, NineStar::LucBach)
    );

    // Đông Chí 2017 began on 21 December in Vietnam and on 22 December in China.
    let dong_chi_2017 = js(&SolarDateTime {
        year: 2017,
        month: 12,
        day: 21,
        hour: 12,
        minute: 0,
        second: 0,
    });
    let vietnam: FlyingStar = rust(hour_nine_star(dong_chi_2017.clone(), None));
    let china: FlyingStar = rust(hour_nine_star(dong_chi_2017, Some(8.0)));
    assert_eq!((vietnam.forward, china.forward), (true, false));

    // The turning point near Hạ Chí 2077 is 22 May in Vietnam and 21 July in China.
    let june_2077 = js(&SolarDate {
        year: 2077,
        month: 6,
        day: 1,
    });
    let vietnam: FlyingStar = rust(day_nine_star(june_2077.clone(), None));
    let china: FlyingStar = rust(day_nine_star(june_2077, Some(8.0)));
    assert_eq!((vietnam.forward, china.forward), (false, true));
}

#[wasm_bindgen_test]
fn lo_shu_grid_flies_forward_unless_told_otherwise() {
    let omitted: LoShuGrid = rust(generate_loshu_grid(variant("CuuTu"), None));
    let forward: LoShuGrid = rust(generate_loshu_grid(variant("CuuTu"), Some(true)));
    assert!(omitted.forward);
    assert_eq!(omitted, forward);
    let rows: [[NineStar; 3]; 3] = rust(lo_shu_rows(js(&omitted)));
    assert_eq!(numbers(rows), [[8, 4, 6], [7, 9, 2], [3, 5, 1]]);

    let backward: LoShuGrid = rust(generate_loshu_grid(variant("TamBich"), Some(false)));
    assert!(!backward.forward);
    let rows: [[NineStar; 3]; 3] = rust(lo_shu_rows(js(&backward)));
    assert_eq!(numbers(rows), [[4, 8, 6], [5, 3, 1], [9, 7, 2]]);
}

#[wasm_bindgen_test]
fn lo_shu_star_at_reads_a_palace() {
    let grid = generate_loshu_grid(variant("CuuTu"), None).unwrap();
    let center: NineStar = rust(lo_shu_star_at(grid.clone(), variant("TrungCung")));
    let south: NineStar = rust(lo_shu_star_at(grid.clone(), variant("Ly")));
    let north_west: NineStar = rust(lo_shu_star_at(grid, variant("CanKim")));
    assert_eq!(
        (center, south, north_west),
        (NineStar::CuuTu, NineStar::TuLuc, NineStar::NhatBach)
    );
}

#[wasm_bindgen_test]
fn center_palace_has_no_trigram_or_direction() {
    assert!(palace_trigram(variant("TrungCung")).unwrap().is_null());
    assert!(palace_direction(variant("TrungCung")).unwrap().is_null());

    let trigram: Trigram = rust(palace_trigram(variant("Ly")));
    let direction: CompassDirection = rust(palace_direction(variant("Ly")));
    assert_eq!((trigram, direction), (Trigram::Ly, CompassDirection::Nam));
}

#[wasm_bindgen_test]
fn results_use_camel_case_fields_and_variant_names() {
    assert_eq!(
        stringify(&kua_info(1990, variant("Female")).unwrap()),
        concat!(
            r#"{"birthLunarYear":1990,"gender":"Female","rawNumber":5,"number":8,"#,
            r#""trigram":"CanTho","group":"TayTu"}"#
        )
    );
    assert_eq!(
        stringify(&generate_loshu_grid(variant("NguHoang"), None).unwrap()),
        concat!(
            r#"{"forward":true,"kham":"NhatBach","khon":"NhiHac","chan":"TamBich","#,
            r#""ton":"TuLuc","trungCung":"NguHoang","canKim":"LucBach","doai":"ThatXich","#,
            r#""canTho":"BatBach","ly":"CuuTu"}"#
        )
    );
    assert_eq!(
        stringify(&nine_stars(js(&VECTOR_2), None, JsValue::UNDEFINED).unwrap()),
        concat!(
            r#"{"nguyen":"HaNguyen","van":"ThatXich","year":"CuuTu","month":"NhatBach","#,
            r#""day":{"star":"CuuTu","forward":true},"#,
            r#""hour":{"star":"TamBich","forward":false}}"#
        )
    );
    assert_eq!(
        stringify(&eight_mansions_chart(variant("Ly")).unwrap()),
        concat!(
            r#"[{"direction":"Bac","star":"DienNien"},{"direction":"DongBac","star":"HoaHai"},"#,
            r#"{"direction":"Dong","star":"SinhKhi"},{"direction":"DongNam","star":"ThienY"},"#,
            r#"{"direction":"Nam","star":"PhucVi"},{"direction":"TayNam","star":"LucSat"},"#,
            r#"{"direction":"Tay","star":"NguQuy"},{"direction":"TayBac","star":"TuyetMenh"}]"#
        )
    );
}

#[wasm_bindgen_test]
fn trigram_attributes_and_names() {
    assert_eq!(trigram_name(variant("CanKim")).unwrap(), "Càn");
    assert_eq!(trigram_name(variant("CanTho")).unwrap(), "Cấn");
    assert_eq!(trigram_han_name(variant("CanTho")).unwrap(), "艮");
    assert_eq!(trigram_number(variant("CanTho")).unwrap(), 8);

    let element: FiveElements = rust(trigram_element(variant("CanTho")));
    let group: TrigramGroup = rust(trigram_group(variant("CanTho")));
    let direction: CompassDirection = rust(trigram_direction(variant("CanTho")));
    let palace: Palace = rust(trigram_palace(variant("CanTho")));
    assert_eq!(
        (element, group, direction, palace),
        (
            FiveElements::Tho,
            TrigramGroup::TayTu,
            CompassDirection::DongBac,
            Palace::CanTho
        )
    );

    let element: FiveElements = rust(trigram_element(variant("Ly")));
    assert_eq!(element, FiveElements::Hoa);

    assert_eq!(trigram_group_name(variant("DongTu")).unwrap(), "Đông Tứ");
    assert_eq!(
        compass_direction_name(variant("TayBac")).unwrap(),
        "Tây Bắc"
    );
}

#[wasm_bindgen_test]
fn palace_attributes_and_names() {
    assert_eq!(palace_name(variant("TrungCung")).unwrap(), "Trung Cung");
    assert_eq!(palace_han_name(variant("TrungCung")).unwrap(), "中");
    assert_eq!(palace_number(variant("TrungCung")).unwrap(), 5);
    assert_eq!(palace_number(variant("CanKim")).unwrap(), 6);

    let element: FiveElements = rust(palace_element(variant("TrungCung")));
    assert_eq!(element, FiveElements::Tho);
    let element: FiveElements = rust(palace_element(variant("Ly")));
    assert_eq!(element, FiveElements::Hoa);
    let element: FiveElements = rust(palace_element(variant("Kham")));
    assert_eq!(element, FiveElements::Thuy);
}

#[wasm_bindgen_test]
fn star_attributes_and_names() {
    let star = || variant("NguQuy");
    assert_eq!(eight_mansions_star_name(star()).unwrap(), "Ngũ Quỷ");
    assert_eq!(eight_mansions_star_han_name(star()).unwrap(), "五鬼");
    assert_eq!(
        eight_mansions_star_corresponding_name(star()).unwrap(),
        "Liêm Trinh"
    );
    let luck: Luck = rust(eight_mansions_star_luck(star()));
    assert_eq!(luck, Luck::Hung);

    let star = || variant("TamBich");
    assert_eq!(nine_star_name(star()).unwrap(), "Tam Bích Mộc");
    assert_eq!(nine_star_han_name(star()).unwrap(), "三碧木");
    assert_eq!(nine_star_number(star()).unwrap(), 3);
    assert_eq!(nine_star_color_name(star()).unwrap(), "Xanh biếc");
    assert_eq!(nine_star_xuan_kong_name(star()).unwrap(), "Lộc Tồn");
    let element: FiveElements = rust(nine_star_element(star()));
    let palace: Palace = rust(nine_star_home_palace(star()));
    let luck: Luck = rust(nine_star_luck(star()));
    assert_eq!(
        (element, palace, luck),
        (FiveElements::Moc, Palace::Chan, Luck::Hung)
    );

    let element: FiveElements = rust(nine_star_element(variant("CuuTu")));
    assert_eq!(element, FiveElements::Hoa);

    assert_eq!(tam_nguyen_name(variant("HaNguyen")).unwrap(), "Hạ Nguyên");
    assert_eq!(five_elements_name(variant("Thuy")).unwrap(), "Thủy");
}

#[wasm_bindgen_test]
fn rejects_unknown_boundaries() {
    for boundary in [
        variant("Tet"),
        variant("lapxuanday"),
        variant(""),
        JsValue::from_f64(2.0),
        JsValue::TRUE,
    ] {
        assert!(nine_stars(js(&VECTOR_2), None, boundary.clone()).is_err());
        assert!(year_nine_star(js(&VECTOR_2), None, boundary.clone()).is_err());
        assert!(month_nine_star(js(&VECTOR_2), None, boundary).is_err());
    }
}

#[wasm_bindgen_test]
fn rejects_invalid_feng_shui_input() {
    assert!(kua_info(1990, variant("Nam")).is_err());
    assert!(house_trigram(variant("North")).is_err());
    assert!(trigram_name(variant("Can")).is_err());
    assert!(palace_trigram(variant("Center")).is_err());
    assert!(generate_loshu_grid(variant("Cuu"), None).is_err());
    assert!(lo_shu_rows(js(&TET_2024)).is_err());

    let february_30 = SolarDate {
        year: 2024,
        month: 2,
        day: 30,
    };
    assert!(day_nine_star(js(&february_30), None).is_err());
    assert!(hour_nine_star(js(&VECTOR_2), Some(99.0)).is_err());
    assert!(nine_stars(js(&VECTOR_2), Some(99.0), JsValue::UNDEFINED).is_err());
}
