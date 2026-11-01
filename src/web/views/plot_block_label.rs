//! French reason why a plot card can't be played.

use crate::engine::plots::PlotBlock;
use crate::game_data::plot::PlotRequirement;

/// French sentence telling why a plot card can't be played.
pub fn plot_block_label(block: PlotBlock) -> String {
    match block {
        PlotBlock::Influence { required, pool } => format!(
            "Il faut {required} influence(s) dans la réserve des Ténèbres : vous en avez {pool}."
        ),
        PlotBlock::Requirement(requirement) => requirement_label(requirement).to_owned(),
        PlotBlock::NoLocation => "Aucun lieu ne peut recevoir cette intrigue.".to_owned(),
    }
}

fn requirement_label(requirement: PlotRequirement) -> &'static str {
    match requirement {
        PlotRequirement::MinionInErebor => "Il faut un sbire à Erebor.",
        PlotRequirement::MonsterInOsgiliath => "Il faut un monstre à Osgiliath.",
        PlotRequirement::InfluenceNearEdoras => {
            "Il faut 6 influences autour d'Edoras, ou l'intrigue de la chute de Saroumane en jeu."
        }
        PlotRequirement::InfluenceInIsengard => "Il faut 3 influences à Isengard.",
        PlotRequirement::InfluenceNearMountGundabad => {
            "Il faut 8 influences autour du mont Gundabad."
        }
        PlotRequirement::RequireMobNearWoddenLand => {
            "Il faut un monstre ou un sbire près du Royaume des Bois."
        }
    }
}
