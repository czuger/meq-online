//! French names of the Mission cards.

use crate::model::mission::{HeroesMission, SauronMission};

/// French name of one of Sauron's Mission cards.
pub fn sauron_mission_name(mission: SauronMission) -> &'static str {
    match mission {
        SauronMission::HisDarkThrone => "Son trône ténébreux",
        SauronMission::WhereTheShadowsLie => "Où s'étendent les Ombres",
        SauronMission::InDarknessBindThem => "Et dans les Ténèbres les lier",
        SauronMission::ToRuleThemAll => "Pour les gouverner tous",
        SauronMission::ToFindThemAll => "Pour les trouver",
    }
}

/// French name of one of the heroes' Mission cards.
pub fn heroes_mission_name(mission: HeroesMission) -> &'static str {
    match mission {
        HeroesMission::SpearOfTheWest => "La Lance de l'Ouest",
        HeroesMission::NobleBlood => "Sang noble",
        HeroesMission::MinasMorgulKeptAtBay => "Minas Morgul tenue en respect",
        HeroesMission::AgainstTheShadow => "Contre l'Ombre",
        HeroesMission::IsildursSecret => "Le Secret d'Isildur",
    }
}
