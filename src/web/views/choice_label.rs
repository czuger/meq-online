//! French labels of choice options and targets.

use crate::engine::choices::parse_target;
use crate::game_data::GameData;
use crate::model::game::Game;
use crate::web::views::pictures::{corruption_card, hero_card, image, plot_card, shadow_card};

/// Label of an option code.
pub fn option_label(code: &str) -> &'static str {
    match code {
        "pass" => "Ne rien faire",
        "spend_favor" => "Dépenser 1 faveur",
        "corruption" => "Recevoir 1 carte de corruption",
        "remove_monsters" => "Retirer des jetons monstre",
        "hurt_minion" => "Infliger des dégâts à un serviteur",
        "place" => "Appliquer l'effet de la carte",
        "give" => "Donner la carte à un héros",
        "choose_hero" => "Choisir un héros",
        "accept" => "Accepter la carte de corruption et la récompense",
        "refuse" => "Refuser : Sauron pioche une carte",
        "move" => "Se déplacer",
        "remove_tokens" => "Retirer de l'influence et des jetons monstre",
        "remove_all" => "Retirer toute l'influence et tous les jetons monstre",
        "place_favor" => "Placer une faveur",
        "discard_character" => "Défausser un personnage",
        "discard_favor" => "Défausser une faveur",
        "move_character" => "Déplacer un personnage",
        "place_monsters" => "Placer des jetons monstre",
        "discard_plot" => "Défausser une intrigue",
        "move_minion" => "Déplacer un serviteur",
        "keep_skill" => "Garder une carte de compétence",
        other => threat_label(other),
    }
}

/// Labels of threats, perils and damage.
fn threat_label(code: &str) -> &'static str {
    match code {
        "attack" => "Attaquer le héros avec un monstre ou un serviteur",
        "peril" => "Infliger un péril",
        "allow_combat" => "Accepter le combat",
        "pick_peril" => "Choisir la carte Péril",
        "pay_damage" => "Perdre ces cartes de la main (le reste vient de la réserve de vie)",
        "discard_shields" => "Défausser ces cartes pour réduire les dégâts",
        "damage" => "Subir les dégâts",
        "sauron_draws" => "Sauron pioche 1 carte Intrigue et 1 carte des Ténèbres",
        "advance_corruption" => "Le marqueur de la corruption avance d'une case",
        "lose_favor" => "Perdre 2 faveurs",
        "give_corruption" => "Donner cette carte de corruption au héros",
        "fight" => "Combattre",
        "minion" => "Choisir le serviteur qui combat",
        "discard_cards" => "Défausser ces cartes",
        "place_influence" => "Placer 1 influence sur ces lieux",
        "place_monster" => "Placer un jeton monstre",
        "draw_plot" => "Piocher 1 carte Intrigue",
        "send_to_haven" => "Envoyer le héros au refuge le plus proche",
        other => shadow_label(other),
    }
}

/// Labels of Shadow card and plot decisions.
fn shadow_label(code: &str) -> &'static str {
    match code {
        "play_shadow" => "Jouer une carte des Ténèbres",
        "damage_per_corruption" => "Infliger 5 dégâts par carte de corruption du héros",
        "damage_and_corruption" => "Infliger 3 dégâts et donner 1 carte de corruption",
        "move_hero" => "Déplacer un héros vers un lieu adjacent",
        "influence_on_hero" => "Choisir un héros et placer 1 influence sur son lieu",
        "mordor_influence" => "Placer 2 influences sur chaque lieu du Mordor",
        "plot_top" => "Laisser la carte sur la pioche Intrigue",
        "plot_bottom" => "Mettre la carte sous la pioche Intrigue",
        "take_card" => "Prendre cette carte en main",
        "take_plot" => "Prendre l'intrigue en main",
        "play_plot" => "Jouer l'intrigue immédiatement",
        "pick_minion" => "Choisir le serviteur",
        "place_minion" => "Placer le serviteur sur ce lieu",
        "choose_heroes" => "Choisir deux héros",
        "redistribute" => {
            "Cartes de corruption données au premier héros (les autres vont au second)"
        }
        "corrupt_hero" => "Donner 1 carte de corruption à ce héros",
        "plot_location" => "Choisir le lieu affecté par l'intrigue",
        "champion" => "Choisir le champion",
        "advance_marker" => "Choisir le marqueur de Sauron qui avance (héros vaincu)",
        other => reward_label(other),
    }
}

/// Labels of the rewards of event cards in play.
fn reward_label(code: &str) -> &'static str {
    match code {
        "remove_adjacent_influence" => "Retirer 1 influence de chaque lieu adjacent",
        "advance_heroes" => "Dépenser des faveurs pour avancer le marqueur des héros",
        "discard_corruption" => "Défausser une carte de corruption",
        "gain_fortitude" => "Gagner 1 niveau d'endurance",
        "favor" => "Gagner des faveurs",
        "training" => "Recevoir un entraînement",
        "boat" => "Gagner l'objet Bateau",
        "shadow_pool_loss" => "Retirer de l'influence de la réserve des Ténèbres",
        "move_anywhere" => "Se déplacer vers n'importe quel lieu",
        "damage_to_rest" => "Mélanger la réserve de blessures dans la réserve de repos",
        "gandalf_to_influence" => "Placer Gandalf sur un lieu avec de l'influence",
        "boat_and_move" => "Gagner l'objet Bateau (et pouvoir se déplacer)",
        "favor_and_remove_monster" => "Gagner des faveurs (et pouvoir retirer un jeton monstre)",
        "gandalf_far_away" => "Placer Gandalf au loin et retirer 1 influence de la réserve",
        "retreat_corruption" => "Dépenser des faveurs pour reculer le marqueur de la corruption",
        "favor_and_clear_monsters" => "Gagner des faveurs et retirer les monstres proches",
        _ => "Option inconnue",
    }
}

/// Label of a target code. Monster tokens stay hidden from the heroes.
pub fn target_label(code: &str, game: &Game, data: &GameData, sauron: bool) -> String {
    let (kind, values) = parse_target(code);
    let first = values.first().copied().unwrap_or_default();
    let second = values.get(1).copied().unwrap_or_default();
    let location = |code: &str| data.locations.name(code).to_owned();
    let character = |code: &str| {
        data.character(code)
            .map_or(code, |c| c.name.as_str())
            .to_owned()
    };
    let mob = |id: &str| {
        id.parse::<i64>()
            .ok()
            .and_then(|id| game.mobs.iter().find(|m| m.id == Some(id)))
    };
    match kind {
        "location" => location(first),
        "influence" => format!("Influence à {} ({second})", location(first)),
        "favor" => format!("Faveur à {}", location(first)),
        "monster" => mob(first).map_or_else(String::new, |m| {
            let name = if sauron {
                m.name.as_str()
            } else {
                "Jeton monstre"
            };
            format!("{name} à {}", location(&m.location))
        }),
        "minion" => mob(first).map_or_else(String::new, |m| {
            format!("{} à {}", m.name, location(&m.location))
        }),
        "character" => match game.board.characters.get(first) {
            Some(at) => format!("{} ({})", character(first), location(at)),
            None => character(first),
        },
        "character_to" => format!("{} vers {}", character(first), location(second)),
        "hero" => first
            .parse::<i64>()
            .ok()
            .and_then(|id| game.hero(id).ok())
            .map_or_else(String::new, |h| {
                data.heroes
                    .get(&h.code)
                    .map_or(h.code.clone(), |d| d.name.clone())
            }),
        "corruption" => first
            .parse()
            .ok()
            .and_then(|c| data.corruption_card(c).ok())
            .map_or_else(String::new, |c| c.name.clone()),
        "skill" => first
            .parse()
            .ok()
            .and_then(|c| data.skills.get(&c))
            .map_or_else(String::new, |s| s.name.clone()),
        "plot" => {
            let hero = target_label(&format!("hero:{second}"), game, data, sauron);
            format!("Intrigue de l'emplacement {first}, payée par {hero}")
        }
        "hand" => hand_card_name(first, second, game, data).unwrap_or_default(),
        "peril" => first
            .parse()
            .ok()
            .and_then(|c| data.peril(c).ok())
            .map_or_else(String::new, |p| format!("{} : {}", p.name_fr, p.text_fr)),
        "minion_to" => {
            let name = mob(first).map_or_else(String::new, |m| m.name.clone());
            format!("{name} vers {}", location(second))
        }
        _ => shadow_target_label(code, game, data, sauron),
    }
}

/// Labels of the targets of Shadow card decisions and of story markers.
fn shadow_target_label(code: &str, game: &Game, data: &GameData, sauron: bool) -> String {
    let (kind, values) = parse_target(code);
    let first = values.first().copied().unwrap_or_default();
    match kind {
        "hero_to" => {
            let hero = target_label(&format!("hero:{first}"), game, data, sauron);
            let to = values.get(1).copied().unwrap_or_default();
            format!("{hero} vers {}", data.locations.name(to))
        }
        "defeated_minion" => data
            .mob(first)
            .map_or_else(|_| first.to_owned(), |m| format!("{} (vaincu)", m.name)),
        "shadow" => first
            .parse()
            .ok()
            .and_then(|c| data.shadow(c).ok())
            .map_or_else(String::new, |c| format!("{} : {}", c.name_fr, c.text_fr)),
        "plot_card" => "Carte Intrigue".to_owned(),
        "marker" => match first {
            "ring" => "Marqueur de l'Anneau",
            "conquest" => "Marqueur de la conquête",
            "corruption" => "Marqueur de la corruption",
            _ => "Marqueur inconnu",
        }
        .to_owned(),
        _ => code.to_owned(),
    }
}

/// Name of the card at `index` in the hand of hero `hero_id`.
fn hand_card_name(hero_id: &str, index: &str, game: &Game, data: &GameData) -> Option<String> {
    let hero = game.hero(hero_id.parse().ok()?).ok()?;
    let card = *hero.hand.get(index.parse::<usize>().ok()?)?;
    let def = data.hero_card(&hero.code, card).ok()?;
    Some(format!("{} (défense {})", def.name, def.defense))
}

/// Picture of a card target (a Skill, Corruption or hand card), if it has one.
pub fn target_picture(code: &str, game: &Game, data: &GameData) -> Option<String> {
    let (kind, values) = parse_target(code);
    if kind == "hand" {
        let hero = game.hero(values.first()?.parse().ok()?).ok()?;
        let card = *hero.hand.get(values.get(1)?.parse::<usize>().ok()?)?;
        return Some(hero_card(data, &hero.code, card).pic);
    }
    let card = values.first()?.parse().ok()?;
    match kind {
        "skill" => data
            .skills
            .get(&card)
            .map(|s| image(&format!("hero_cards/actions/{}", s.pic_path))),
        "corruption" => Some(corruption_card(card).pic),
        "shadow" => Some(shadow_card(card).pic),
        "plot_card" => Some(plot_card(card).pic),
        _ => None,
    }
}
