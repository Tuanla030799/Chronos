//! Runs in Node.js: `wasm-pack test --node crates/astro-wasm`.
#![cfg(target_arch = "wasm32")]

use astro_core::almanac::Luck;
use astro_core::fengshui::{
    CompassDirection, DirectionStar, EightMansionsStar, FlyingStar, KuaCompatibility, KuaInfo,
    LoShuGrid, NineStar, NineStars, Palace, TamNguyen, Trigram, TrigramGroup,
};
use astro_core::tuvi::{
    FiveElementsBureau, FourTransformations, StarBrightness, StarCategory, StarNature, TuViChart,
    TuViPalace, TuViStar,
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
    solar_terms_in_year, solar_to_lunar, star_at, star_of_year, star_towards, stem_name, tai_sui,
    tai_sui_luck, tai_sui_name, tam_nguyen_name, trigram_direction, trigram_element, trigram_group,
    trigram_group_name, trigram_han_name, trigram_name, trigram_number, trigram_palace, truc_name,
    true_solar_time, van_of_year, year_nine_star,
};
use astro_wasm::{
    five_elements_bureau_element, five_elements_bureau_name, five_elements_bureau_number,
    four_transformation_name, four_transformation_nature, four_transformation_star,
    menh_cuc_relation_name, star_brightness_mark, star_brightness_name, star_nature_name,
    tuvi_all_stars, tuvi_chart, tuvi_chart_from_lunar, tuvi_dai_han_palace, tuvi_palace_role_name,
    tuvi_polarity_name, tuvi_star_brightness, tuvi_star_category, tuvi_star_name, tuvi_star_nature,
    tuvi_tieu_han_palace, yin_yang_harmony_name,
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

    /// `JSON.parse`: an options object as a JavaScript caller writes it.
    #[wasm_bindgen(js_namespace = JSON, js_name = parse)]
    fn js_sys_object(text: &str) -> JsValue;
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

    let reading: astro_core::almanac::TaiSuiReading = rust(tai_sui(1984, 2026));
    assert_eq!(reading.relations, [astro_core::almanac::TaiSui::Xung]);
    assert_eq!(
        tai_sui_name(js(&reading.relations[0])).unwrap(),
        "Xung Thái Tuế"
    );
    let luck: astro_core::almanac::Luck =
        rust(tai_sui_luck(js(&astro_core::almanac::TaiSui::LucHop)));
    assert_eq!(luck, astro_core::almanac::Luck::Cat);
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

/// Spec vector 3: Dương Nam in a leap month.
const TUVI_VECTOR_3: SolarDateTime = SolarDateTime {
    year: 2020,
    month: 6,
    day: 10,
    hour: 10,
    minute: 30,
    second: 0,
};

const BOOK_CHART_2: LunarDate = LunarDate {
    year: 1891,
    month: 6,
    day: 6,
    is_leap_month: false,
};

#[wasm_bindgen_test]
fn tuvi_chart_takes_optional_options() {
    let chart: TuViChart = rust(tuvi_chart(
        js(&TUVI_VECTOR_3),
        variant("Male"),
        JsValue::UNDEFINED,
    ));
    assert_eq!(chart.effective_month, 5);
    assert_eq!(chart.palaces.len(), 12);
    assert_eq!(chart.bureau, FiveElementsBureau::HoaLuc);

    let same: TuViChart = rust(tuvi_chart(
        js(&TUVI_VECTOR_3),
        variant("Male"),
        JsValue::NULL,
    ));
    assert_eq!(same, chart);

    let options = js_sys_object(r#"{"leapMonthRule":"SameAsCurrentMonth"}"#);
    let other: TuViChart = rust(tuvi_chart(js(&TUVI_VECTOR_3), variant("Male"), options));
    assert_eq!(other.effective_month, 4);
}

#[wasm_bindgen_test]
fn tuvi_chart_is_camel_case_for_javascript() {
    let result = tuvi_chart_from_lunar(
        js(&BOOK_CHART_2),
        variant("Mao"),
        variant("Male"),
        JsValue::UNDEFINED,
    )
    .unwrap();
    let text = stringify(&result);
    for key in [
        r#""menhBranch":"Thin""#,
        r#""bureau":"ThuyNhi""#,
        r#""polarity":"AmNam""#,
        r#""effectiveTime":null"#,
        r#""dayCanChi""#,
        r#""mainStars""#,
        r#""minorStars""#,
        r#""lifeStage""#,
        r#""daiHanStartAge""#,
        r#""tieuHanYearBranch""#,
        r#""hasTuan""#,
        r#""transformation":"HoaQuyen""#,
        r#""brightness":"Vuong""#,
        r#""brightness":null"#,
    ] {
        assert!(text.contains(key), "{key} missing from {text}");
    }
}

#[wasm_bindgen_test]
fn tuvi_limits_take_a_chart_back() {
    let chart = tuvi_chart_from_lunar(
        js(&BOOK_CHART_2),
        variant("Mao"),
        variant("Male"),
        JsValue::UNDEFINED,
    )
    .unwrap();
    let dai_han: TuViPalace = rust(tuvi_dai_han_palace(chart.clone(), 12));
    assert_eq!(dai_han.dai_han_start_age, 12);
    let tieu_han: TuViPalace = rust(tuvi_tieu_han_palace(chart.clone(), 1));
    assert_eq!(tieu_han.tieu_han_first_age, 1);
    let none: Option<TuViPalace> = rust(tuvi_dai_han_palace(chart, 1));
    assert_eq!(none, None);
}

#[wasm_bindgen_test]
fn tuvi_helpers_delegate_to_the_core() {
    let stars: Vec<TuViStar> = rust(tuvi_all_stars());
    assert_eq!(stars.len(), 105);
    assert_eq!(tuvi_star_name(variant("QuanPhuBacSy")).unwrap(), "Quan Phủ");
    let category: StarCategory = rust(tuvi_star_category(variant("TuVi")));
    assert_eq!(category, StarCategory::ChinhTinh);
    let nature: StarNature = rust(tuvi_star_nature(variant("KinhDuong")));
    assert_eq!(nature, StarNature::Hung);
    let brightness: Option<StarBrightness> =
        rust(tuvi_star_brightness(variant("TuVi"), variant("Ngo")));
    assert_eq!(brightness, Some(StarBrightness::Mieu));
    let none: Option<StarBrightness> =
        rust(tuvi_star_brightness(variant("LocTon"), variant("Ngo")));
    assert_eq!(none, None);
    let star: TuViStar = rust(four_transformation_star(
        variant("HoaKhoa"),
        variant("Nham"),
    ));
    assert_eq!(star, TuViStar::TaPhu);
    let nature: StarNature = rust(four_transformation_nature(variant("HoaKy")));
    assert_eq!(nature, StarNature::Hung);
    assert_eq!(star_brightness_name(variant("Mieu")).unwrap(), "Miếu");
    assert_eq!(star_brightness_mark(variant("Dac")).unwrap(), "Đ");
    assert_eq!(star_nature_name(variant("Cat")).unwrap(), "Cát");
    assert_eq!(
        four_transformation_name(variant("HoaLoc")).unwrap(),
        "Hóa Lộc"
    );
    assert_eq!(tuvi_palace_role_name(variant("PhuMau")).unwrap(), "Phụ Mẫu");
    assert_eq!(
        five_elements_bureau_name(variant("MocTam")).unwrap(),
        "Mộc Tam Cục"
    );
    assert_eq!(five_elements_bureau_number(variant("HoaLuc")).unwrap(), 6);
    let element: FiveElements = rust(five_elements_bureau_element(variant("KimTu")));
    assert_eq!(element, FiveElements::Kim);
    assert_eq!(tuvi_polarity_name(variant("AmNu")).unwrap(), "Âm Nữ");
    assert_eq!(
        yin_yang_harmony_name(variant("ThuanLy")).unwrap(),
        "Thuận lý"
    );
    assert_eq!(
        menh_cuc_relation_name(variant("BinhHoa")).unwrap(),
        "Bình hòa"
    );
}

#[wasm_bindgen_test]
fn rejects_invalid_tuvi_input() {
    assert!(tuvi_chart(js(&TUVI_VECTOR_3), variant("Nam"), JsValue::UNDEFINED).is_err());
    let unknown_rule = js_sys_object(r#"{"leapMonthRule":"Split"}"#);
    assert!(tuvi_chart(js(&TUVI_VECTOR_3), variant("Male"), unknown_rule).is_err());
    let bad_zone = js_sys_object(r#"{"tzOffset":99}"#);
    assert!(tuvi_chart(js(&TUVI_VECTOR_3), variant("Male"), bad_zone).is_err());
    let bad_date = LunarDate {
        month: 13,
        ..BOOK_CHART_2
    };
    assert!(
        tuvi_chart_from_lunar(
            js(&bad_date),
            variant("Mao"),
            variant("Male"),
            JsValue::UNDEFINED
        )
        .is_err()
    );
    assert!(tuvi_star_name(variant("Sao")).is_err());
}

#[wasm_bindgen_test]
fn palace_queries_and_gates_work_from_javascript() {
    use astro_wasm::{gate_matches, tuvi_chart_from_lunar, tuvi_palace_query};
    let lunar = js_sys_object(r#"{ "year": 1931, "month": 11, "day": 30, "isLeapMonth": false }"#);
    let chart = tuvi_chart_from_lunar(lunar, variant("Dan"), variant("Male"), JsValue::UNDEFINED);
    let query = tuvi_palace_query(chart.unwrap(), variant("Dan")).unwrap();
    let text = stringify(&query);
    assert!(text.contains(r#""keys":["tuvi.palace/QuanLoc""#), "{text}");
    assert!(
        text.contains(r#""QuanLoc/ThaiDuong","QuanLoc/CuMon""#),
        "{text}"
    );
    let gates = js_sys_object(r#"[{ "all": ["solo"], "any": [], "none": [] }]"#);
    assert!(gate_matches(gates.clone(), js_sys_object(r#"["solo"]"#)).unwrap());
    assert!(!gate_matches(gates, js_sys_object("[]")).unwrap());
}

#[wasm_bindgen_test]
fn nine_star_phases_and_keys_work_from_javascript() {
    use astro_wasm::{nine_star_key, nine_star_phase, nine_star_phase_is_auspicious};
    let phase = nine_star_phase(variant("BatBach"), variant("CuuTu")).unwrap();
    assert_eq!(stringify(&phase), r#""ThoaiKhi""#);
    assert!(nine_star_phase_is_auspicious(phase).unwrap());
    assert_eq!(
        nine_star_key(variant("CuuTu")).unwrap(),
        "fengshui.nine_star/CuuTu"
    );
}

#[wasm_bindgen_test]
fn bazi_item_keys_and_elements_work_from_javascript() {
    use astro_wasm::{
        bazi_day_master_key, bazi_khoi_canh_key, bazi_khong_vong_key, bazi_life_stage_key,
        bazi_na_yin_key, bazi_shen_sha_key, bazi_ten_god_key, branch_element, na_yin_element,
        stem_element,
    };
    assert_eq!(
        bazi_day_master_key(variant("Giap")).unwrap(),
        "bazi.day_master/Giap"
    );
    assert_eq!(
        bazi_ten_god_key(variant("ThucThan")).unwrap(),
        "bazi.ten_god/ThucThan"
    );
    assert_eq!(
        bazi_life_stage_key(variant("TruongSinh")).unwrap(),
        "bazi.life_stage/TruongSinh"
    );
    assert_eq!(
        bazi_na_yin_key(variant("PhuDangHoa")).unwrap(),
        "bazi.nayin/PhuDangHoa"
    );
    assert_eq!(
        bazi_shen_sha_key(variant("DichMa")).unwrap(),
        "bazi.shen_sha/DichMa"
    );
    assert_eq!(bazi_khong_vong_key(), "bazi.shen_sha/KhongVong");
    assert_eq!(bazi_khoi_canh_key(), "bazi.shen_sha/KhoiCanh");
    assert_eq!(
        astro_wasm::eight_use_name(variant("BanLamViec")).unwrap(),
        "Bàn làm việc"
    );
    assert_eq!(
        stringify(&stem_element(variant("Giap")).unwrap()),
        r#""Moc""#
    );
    assert_eq!(
        stringify(&branch_element(variant("Mui")).unwrap()),
        r#""Tho""#
    );
    assert_eq!(
        stringify(&na_yin_element(variant("PhuDangHoa")).unwrap()),
        r#""Hoa""#
    );
}

#[wasm_bindgen_test]
fn tuvi_item_keys_trines_and_elements_work_from_javascript() {
    use astro_wasm::{
        branch_clash, branch_trine, tuvi_palace_key, tuvi_star_brightness_key, tuvi_star_element,
        tuvi_star_key, tuvi_transformation_key,
    };
    assert_eq!(
        tuvi_palace_key(variant("Menh")).unwrap(),
        "tuvi.palace/Menh"
    );
    assert_eq!(tuvi_star_key(variant("TuVi")).unwrap(), "tuvi.star/TuVi");
    assert_eq!(
        tuvi_star_brightness_key(variant("CuMon"), variant("Ham")).unwrap(),
        "tuvi.star_brightness/CuMon/Ham"
    );
    assert_eq!(
        tuvi_transformation_key(variant("HoaLoc")).unwrap(),
        "tuvi.transformation/HoaLoc"
    );
    assert_eq!(
        stringify(&tuvi_star_element(variant("TuVi")).unwrap()),
        r#""Tho""#
    );
    assert_eq!(
        stringify(&tuvi_star_element(variant("LocTon")).unwrap()),
        "null"
    );
    assert_eq!(
        stringify(&branch_trine(variant("Ty")).unwrap()),
        r#"["Than","Ty","Thin"]"#
    );
    assert_eq!(stringify(&branch_clash(variant("Ty")).unwrap()), r#""Ngo""#);
}
