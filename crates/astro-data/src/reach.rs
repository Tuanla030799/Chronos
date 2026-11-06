//! Check V7: whether a condition can ever hold, on the 144 states of the Chính Tinh.
//!
//! Twelve positions of Tử Vi fix all 14 Chính Tinh, so a palace has only 144 possible states. A
//! condition that fails on all of them is a misreading or a misprint. Only the Chính Tinh and the
//! table of Tứ Hóa are known here: conditions on other stars, or on the year, count as possible.

use crate::model::{StarRef, When};
use astro_core::tuvi::{
    FourTransformations, StarBrightness, StarCategory, TuViStar, chinh_tinh_branches,
};
use astro_core::{Branch, Stem};
use std::collections::BTreeMap;

/// A [`When`] with its groups replaced by stars.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resolved {
    /// The palace is at one of these branches.
    pub branches: Vec<Branch>,
    /// The anchor star has one of these levels.
    pub brightness: Vec<StarBrightness>,
    /// Whether the palace has exactly one Chính Tinh.
    pub solo: Option<bool>,
    /// All sit in the palace.
    pub with_all: Vec<TuViStar>,
    /// None sits in the palace.
    pub not_with: Vec<TuViStar>,
    /// One meets the palace.
    pub meets_any: Vec<TuViStar>,
    /// All meet the palace.
    pub meets_all: Vec<TuViStar>,
    /// None meets the palace.
    pub not_meets: Vec<TuViStar>,
    /// All sit in the Nhị Hợp palace.
    pub paired_with: Vec<TuViStar>,
    /// All sit in the neighbouring palaces.
    pub flanked_by: Vec<TuViStar>,
    /// One of these transformations meets the palace.
    pub transformations: Vec<FourTransformations>,
    /// All these transformations meet the palace.
    pub transformations_all: Vec<FourTransformations>,
    /// One of these transformations sits in the palace.
    pub with_transformations: Vec<FourTransformations>,
}

/// Replaces `@Group` by its stars. The error names what is neither a star nor a group.
pub fn expand(
    stars: &[StarRef],
    groups: &BTreeMap<String, Vec<TuViStar>>,
) -> Result<Vec<TuViStar>, String> {
    let mut expanded = Vec::new();
    for star in stars {
        match star {
            StarRef::Star(star) => expanded.push(*star),
            StarRef::Group(name) => {
                let group = name
                    .strip_prefix('@')
                    .and_then(|id| groups.get(id))
                    .ok_or_else(|| format!("{name} không phải sao hay nhóm sao"))?;
                expanded.extend_from_slice(group);
            }
        }
    }
    Ok(expanded)
}

/// `when` with its star lists expanded.
pub fn resolve(when: &When, groups: &BTreeMap<String, Vec<TuViStar>>) -> Result<Resolved, String> {
    Ok(Resolved {
        branches: when.branches.clone(),
        brightness: when.brightness.clone(),
        solo: when.solo,
        with_all: expand(&when.with_all, groups)?,
        not_with: expand(&when.not_with, groups)?,
        meets_any: expand(&when.meets_any, groups)?,
        meets_all: expand(&when.meets_all, groups)?,
        not_meets: expand(&when.not_meets, groups)?,
        paired_with: expand(&when.paired_with, groups)?,
        flanked_by: expand(&when.flanked_by, groups)?,
        transformations: when.transformations.clone(),
        transformations_all: when.transformations_all.clone(),
        with_transformations: when.with_transformations.clone(),
    })
}

const fn is_main(star: TuViStar) -> bool {
    matches!(star.category(), StarCategory::ChinhTinh)
}

/// A palace on a board fixed by the position of Tử Vi.
#[derive(Debug, Clone, Copy)]
struct State {
    palace: Branch,
    stars: [(TuViStar, Branch); 14],
}

impl State {
    fn at(&self, star: TuViStar) -> Branch {
        self.stars[star as usize].1
    }

    fn main_count(&self) -> usize {
        self.stars
            .iter()
            .filter(|(_, branch)| *branch == self.palace)
            .count()
    }

    fn sits(&self, star: TuViStar) -> bool {
        !is_main(star) || self.at(star) == self.palace
    }

    /// In the palace or facing it, the meaning of "hội hợp" under D7.
    fn meets(&self, star: TuViStar) -> bool {
        let at = self.at(star);
        self.palace.trine().contains(&at) || self.palace.clash() == at
    }

    fn may_meet(&self, star: TuViStar) -> bool {
        !is_main(star) || self.meets(star)
    }

    /// Whether, for some year stem, every transformation of `hoas` lands where `place` allows.
    fn hoas_may_land(
        &self,
        hoas: &[FourTransformations],
        place: fn(&Self, TuViStar) -> bool,
    ) -> bool {
        Stem::ALL
            .into_iter()
            .any(|stem| hoas.iter().all(|hoa| place(self, hoa.star_for(stem))))
    }

    fn holds_transformations(&self, when: &Resolved) -> bool {
        let one_lands = |hoas: &[FourTransformations], place: fn(&Self, TuViStar) -> bool| {
            hoas.is_empty() || hoas.iter().any(|&hoa| self.hoas_may_land(&[hoa], place))
        };
        one_lands(&when.transformations, Self::may_meet)
            && one_lands(&when.with_transformations, Self::sits)
            && self.hoas_may_land(&when.transformations_all, Self::may_meet)
    }

    fn holds(&self, anchor: Option<TuViStar>, no_main: bool, when: &Resolved) -> bool {
        if no_main && self.main_count() != 0 {
            return false;
        }
        if let Some(star) = anchor {
            if is_main(star) && self.at(star) != self.palace {
                return false;
            }
        }
        if !when.branches.is_empty() && !when.branches.contains(&self.palace) {
            return false;
        }
        if !when.brightness.is_empty() {
            let level = anchor.and_then(|star| star.brightness_at(self.palace));
            if !level.is_some_and(|level| when.brightness.contains(&level)) {
                return false;
            }
        }
        if when
            .solo
            .is_some_and(|solo| solo != (self.main_count() == 1))
        {
            return false;
        }
        let neighbours = [
            Branch::from_index(self.palace.index() as i64 - 1),
            Branch::from_index(self.palace.index() as i64 + 1),
        ];
        self.holds_transformations(when)
            && when.with_all.iter().all(|&star| self.sits(star))
            && when
                .not_with
                .iter()
                .all(|&s| !is_main(s) || self.at(s) != self.palace)
            && (when.meets_any.is_empty() || when.meets_any.iter().any(|&s| self.may_meet(s)))
            && when.meets_all.iter().all(|&star| self.may_meet(star))
            && when
                .not_meets
                .iter()
                .all(|&s| !is_main(s) || !self.meets(s))
            && when
                .paired_with
                .iter()
                .all(|&star| !is_main(star) || self.at(star) == self.palace.six_combination())
            && when
                .flanked_by
                .iter()
                .all(|&star| !is_main(star) || neighbours.contains(&self.at(star)))
    }
}

fn states() -> impl Iterator<Item = State> {
    Branch::ALL.into_iter().flat_map(|tu_vi| {
        Branch::ALL.into_iter().map(move |palace| State {
            palace,
            stars: chinh_tinh_branches(tu_vi),
        })
    })
}

fn reachable(anchor: Option<TuViStar>, no_main: bool, when: &Resolved) -> bool {
    states().any(|state| state.holds(anchor, no_main, when))
}

/// What cannot happen in a passage anchored on `anchors` with alternatives `when`: an alternative
/// that no anchor star reaches, or an anchor star that no alternative reaches.
pub fn impossible(anchors: &[TuViStar], no_main: bool, when: &[Resolved]) -> Vec<String> {
    let always = [Resolved::default()];
    let alternatives = if when.is_empty() { &always[..] } else { when };
    let mut problems = Vec::new();
    let candidates: Vec<Option<TuViStar>> = if anchors.is_empty() {
        vec![None]
    } else {
        anchors.iter().copied().map(Some).collect()
    };
    for (index, alternative) in alternatives.iter().enumerate() {
        if !candidates
            .iter()
            .any(|&star| reachable(star, no_main, alternative))
        {
            problems.push(format!("phương án {} không bao giờ xảy ra", index + 1));
        }
    }
    for &star in anchors {
        if !alternatives
            .iter()
            .any(|when| reachable(Some(star), no_main, when))
        {
            problems.push(format!("{} không bao giờ thỏa điều kiện", star.name_vi()));
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;
    use astro_core::Branch::{Dan, Ngo, Than, Thin, Ty};
    use astro_core::tuvi::FourTransformations::{HoaKhoa, HoaLoc, HoaQuyen};
    use astro_core::tuvi::StarBrightness::{Binh, Ham, Mieu};
    use astro_core::tuvi::TuViStar::{
        LocTon, PhaQuan, ThaiAm, ThaiDuong, ThatSat, ThienCo, ThienLuong, ThienPhu, TuVi,
    };

    #[test]
    fn solo_at_ngo_and_thien_phu_in_the_palace_never_happen_together() {
        let both = Resolved {
            solo: Some(true),
            branches: vec![Ngo],
            with_all: vec![ThienPhu],
            ..Resolved::default()
        };
        assert_eq!(impossible(&[TuVi], false, &[both]).len(), 2);
        let either = [
            Resolved {
                solo: Some(true),
                branches: vec![Ngo],
                ..Resolved::default()
            },
            Resolved {
                with_all: vec![ThienPhu],
                ..Resolved::default()
            },
        ];
        assert_eq!(impossible(&[TuVi], false, &either), Vec::<String>::new());
    }

    #[test]
    fn an_alternative_for_one_anchor_star_is_enough() {
        let alternatives = [
            Resolved {
                solo: Some(true),
                branches: vec![Ngo],
                ..Resolved::default()
            },
            Resolved {
                with_all: vec![TuVi, ThienPhu],
                ..Resolved::default()
            },
        ];
        assert!(impossible(&[TuVi, ThienPhu], false, &alternatives).is_empty());
    }

    #[test]
    fn co_nguyet_share_a_palace_at_dan_and_than_but_not_at_thin() {
        let at = |branch| Resolved {
            with_all: vec![ThienCo, ThaiAm],
            branches: vec![branch],
            ..Resolved::default()
        };
        assert!(impossible(&[ThienCo], false, &[at(Dan)]).is_empty());
        assert!(impossible(&[ThienCo], false, &[at(Than)]).is_empty());
        assert!(!impossible(&[ThienCo], false, &[at(Thin)]).is_empty());
    }

    #[test]
    fn thien_luong_and_pha_quan_never_share_a_palace() {
        let when = Resolved {
            with_all: vec![ThienLuong, PhaQuan],
            ..Resolved::default()
        };
        assert!(!impossible(&[ThienLuong], false, &[when]).is_empty());
    }

    #[test]
    fn brightness_must_exist_for_the_star() {
        let level = |level| Resolved {
            brightness: vec![level],
            ..Resolved::default()
        };
        assert!(impossible(&[TuVi], false, &[level(Mieu)]).is_empty());
        assert!(impossible(&[TuVi], false, &[level(Binh)]).is_empty());
        assert!(!impossible(&[TuVi], false, &[level(Ham)]).is_empty());
    }

    #[test]
    fn brightness_and_branch_must_agree() {
        // Tử Vi is Bình at Tý and Miếu at Ngọ.
        let when = |branch, level| Resolved {
            branches: vec![branch],
            brightness: vec![level],
            ..Resolved::default()
        };
        assert!(impossible(&[TuVi], false, &[when(Ty, Binh)]).is_empty());
        assert!(!impossible(&[TuVi], false, &[when(Ty, Mieu)]).is_empty());
    }

    #[test]
    fn conditions_on_other_stars_count_as_possible() {
        let when = Resolved {
            with_all: vec![LocTon],
            meets_any: vec![LocTon],
            ..Resolved::default()
        };
        assert!(impossible(&[TuVi], false, &[when]).is_empty());
        assert!(impossible(&[LocTon], false, &[]).is_empty());
    }

    #[test]
    fn a_palace_without_chinh_tinh_cannot_be_solo_or_hold_one() {
        assert!(impossible(&[], true, &[]).is_empty());
        let solo = Resolved {
            solo: Some(true),
            ..Resolved::default()
        };
        assert!(!impossible(&[], true, &[solo]).is_empty());
        let holds = Resolved {
            with_all: vec![TuVi],
            ..Resolved::default()
        };
        assert!(!impossible(&[], true, &[holds]).is_empty());
    }

    #[test]
    fn sun_and_moon_can_shine_on_a_palace_without_chinh_tinh() {
        let when = Resolved {
            meets_all: vec![ThaiDuong, ThaiAm],
            ..Resolved::default()
        };
        assert!(impossible(&[], true, &[when]).is_empty());
    }

    #[test]
    fn hoa_quyen_never_shares_the_palace_of_that_sat_at_dan() {
        // Thất Sát is alone at Dần, Thân and only Chính Tinh take Hóa Quyền.
        let with = Resolved {
            branches: vec![Dan, Than],
            with_transformations: vec![HoaQuyen],
            ..Resolved::default()
        };
        assert!(!impossible(&[ThatSat], false, &[with]).is_empty());
        let met = Resolved {
            branches: vec![Dan, Than],
            transformations: vec![HoaQuyen],
            ..Resolved::default()
        };
        assert!(impossible(&[ThatSat], false, &[met]).is_empty());
    }

    #[test]
    fn hoa_khoa_can_land_on_a_minor_star_beside_that_sat() {
        let with = Resolved {
            branches: vec![Dan],
            with_transformations: vec![HoaKhoa],
            ..Resolved::default()
        };
        assert!(impossible(&[ThatSat], false, &[with]).is_empty());
    }

    #[test]
    fn the_three_transformations_can_all_meet_a_palace_in_one_year() {
        let all = Resolved {
            transformations_all: vec![HoaKhoa, HoaQuyen, HoaLoc],
            ..Resolved::default()
        };
        assert!(impossible(&[], false, &[all]).is_empty());
    }

    #[test]
    fn a_star_cannot_both_anchor_and_be_excluded() {
        let when = Resolved {
            not_with: vec![TuVi],
            ..Resolved::default()
        };
        assert!(!impossible(&[TuVi], false, &[when]).is_empty());
    }

    #[test]
    fn a_star_always_meets_its_own_palace() {
        let not_met = Resolved {
            not_meets: vec![TuVi],
            ..Resolved::default()
        };
        assert!(!impossible(&[TuVi], false, &[not_met]).is_empty());
    }

    #[test]
    fn thien_phu_never_meets_that_sat_from_the_side() {
        // Thất Sát sits six palaces from Thiên Phủ: always opposite, so always met.
        let apart = Resolved {
            not_meets: vec![ThatSat],
            ..Resolved::default()
        };
        assert!(!impossible(&[ThienPhu], false, &[apart]).is_empty());
        let beside = Resolved {
            flanked_by: vec![ThatSat],
            ..Resolved::default()
        };
        assert!(!impossible(&[ThienPhu], false, &[beside]).is_empty());
    }

    #[test]
    fn the_sun_and_the_moon_can_flank_a_palace() {
        let flanked = Resolved {
            flanked_by: vec![ThaiDuong, ThaiAm],
            ..Resolved::default()
        };
        assert!(impossible(&[], false, &[flanked]).is_empty());
    }
}
