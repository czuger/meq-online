//! French text of turn steps and of the reasons a step couldn't be played.

use crate::model::turn_step::{StepSkip, TurnStep, TurnStop};

/// French name of a step.
pub fn step_label(step: TurnStep) -> &'static str {
    match step {
        TurnStep::Rally => "Ralliement",
        TurnStep::Story => "Histoire",
        TurnStep::Plot => "Intrigue",
        TurnStep::Event => "Événement",
        TurnStep::Action => "Actions",
        TurnStep::HeroDraw => "Pioche des héros",
        TurnStep::Ambush => "Embuscade",
        TurnStep::Rest => "Repos",
        TurnStep::Movement => "Déplacement",
        TurnStep::Exploration => "Exploration",
    }
}

/// French sentence telling why a step couldn't be played.
pub fn skip_text(reason: StepSkip) -> &'static str {
    match reason {
        StepSkip::FirstRound => {
            "Pas de cette étape au premier tour : la partie commence à l'Événement."
        }
        StepSkip::NoInfluenceUnderHeroes => {
            "Aucun héros ne se trouve sur un lieu avec de l'influence."
        }
        StepSkip::EventDeckEmpty => "Il n'y a plus aucune carte événement à tirer.",
        StepSkip::NoMobHere => "Aucun monstre ni sbire sur ce lieu : pas d'embuscade.",
        StepSkip::HavenProtects => "Le héros refuse le combat dans ce refuge : pas d'embuscade.",
        StepSkip::MobHere => "Repos impossible : un monstre ou un sbire se trouve sur ce lieu.",
        StepSkip::PlotForbidsRest => {
            "Repos impossible : une intrigue active l'interdit sur ce lieu."
        }
        StepSkip::ShadowStronghold => "Repos impossible sur une forteresse des Ténèbres.",
        StepSkip::AmbushFought => "Pas de repos : le héros a combattu pendant l'embuscade.",
        StepSkip::EmptyLifePool => "sa réserve de vie est vide.",
        StepSkip::HandFull => "sa main est pleine.",
    }
}

/// French sentence telling why a turn ended early.
pub fn stop_text(reason: TurnStop) -> &'static str {
    match reason {
        TurnStop::CombatNotWon => {
            "Tour interrompu : le héros n'a pas vaincu son adversaire, il ne peut ni continuer son déplacement ni explorer."
        }
        TurnStop::Defeated => "Tour interrompu : le héros est vaincu.",
    }
}
