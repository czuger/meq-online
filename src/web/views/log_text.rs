//! French text of log entries.

use serde_json::Value;

use crate::game_data::GameData;
use crate::model::log_entry::LogEntry;
use crate::web::views::pictures::image;

/// Value of a log parameter as text; hero codes are replaced by hero names.
fn param(entry: &LogEntry, name: &str, data: &GameData) -> String {
    let text = match entry.params.get(name) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    };
    data.heroes.get(&text).map_or(text, |h| h.name.clone())
}

fn count(entry: &LogEntry) -> i64 {
    entry
        .params
        .get("count")
        .and_then(Value::as_i64)
        .unwrap_or(0)
}

/// "1 jeton" / "3 jetons".
fn plural(n: i64, singular: &str, plural: &str) -> String {
    if n == 1 {
        format!("1 {singular}")
    } else {
        format!("{n} {plural}")
    }
}

fn marker_name(code: &str) -> &'static str {
    match code {
        "heroes" => "des héros",
        "ring" => "de l'Anneau",
        "conquest" => "de la conquête",
        "corruption" => "de la corruption",
        _ => "inconnu",
    }
}

fn sauron_action_name(code: &str) -> &'static str {
    match code {
        "place_influence_1" => "Placer 6 influences",
        "place_influence_2" => "Placer 5 influences",
        "place_influence_3" => "Placer 4 influences",
        "draw_cards_1" | "draw_cards_2" => "Piocher 2 cartes",
        "draw_cards_3" => "Piocher 1 carte",
        "command_1" => "Commander 3",
        "command_2" => "Commander 2",
        "command_3" => "Commander 1",
        _ => "action inconnue",
    }
}

/// Name of a Skill card from its number.
fn skill_name(number: &str, data: &GameData) -> String {
    number
        .parse()
        .ok()
        .and_then(|n| data.skills.get(&n))
        .map_or_else(|| number.to_owned(), |s| s.name.clone())
}

/// Sentence describing a log entry.
pub fn log_text(entry: &LogEntry, data: &GameData) -> String {
    let p = |name: &str| param(entry, name, data);
    let n = count(entry);
    let tokens = plural(n, "jeton d'influence", "jetons d'influence");
    match entry.action.as_str() {
        "rally.hero_remove" => format!("{} à {} a retiré {tokens}.", p("hero"), p("location")),
        "rally.auto_remove" => format!("À {}, {tokens} retiré(s).", p("location")),
        "story.advance" => format!(
            "Le marqueur {} avance de {}.",
            marker_name(&p("marker_name")),
            plural(n, "case", "cases")
        ),
        "exploration.get_favor" => format!("Obtient une faveur à {}.", p("location_name")),
        "exploration.encounter_character" => {
            format!(
                "Rencontre {} à {} et obtient 2 faveurs.",
                p("character_name"),
                p("location_name")
            )
        }
        "exploration.plot.remove" => format!("Retire une intrigue à {}.", p("location_name")),
        "peril.pass_trough" => format!(
            "Entre dans un lieu périlleux à {}. Une impression étrange, mais rien ne se passe.",
            p("location")
        ),
        "peril.encounter_orcs_couts" => {
            format!(
                "Croise des éclaireurs orques à {} et reçoit la carte de corruption {}.",
                p("location"),
                p("corruption")
            )
        }
        "peril.shadow_background_memories" => format!(
            "Est hanté par les ombres du passé à {}. Perd une faveur et reçoit la carte de corruption {}.",
            p("location"),
            p("corruption")
        ),
        "defeat.lose_favor" => "Vaincu, perd une faveur.".to_owned(),
        "defeat.lose_item" => "Vaincu, perd un objet.".to_owned(),
        "defeat.haven" => format!("Vaincu, est transporté à {} et récupère.", p("location")),
        "ringwraiths.return" => format!("Les Nazgûl reviennent à {}.", p("location")),
        "combat.undefeated_influence" => format!(
            "{} n'a pas été vaincu : Sauron peut placer {tokens}.",
            p("name")
        ),
        "corruption.isolated" => format!(
            "N'a pas pu parler avec {} à cause de la corruption Isolé.",
            p("character")
        ),
        "corruption.dispairing" => {
            "N'a pas pu prendre toutes les faveurs à cause de la corruption Désespéré.".to_owned()
        }
        "corruption.cowardly" => {
            "A perdu une carte héros au hasard à cause de la corruption Lâche.".to_owned()
        }
        _ => setup_and_combat_log_text(entry, data),
    }
}

fn setup_and_combat_log_text(entry: &LogEntry, data: &GameData) -> String {
    let p = |name: &str| param(entry, name, data);
    let tokens = plural(count(entry), "jeton d'influence", "jetons d'influence");
    match entry.action.as_str() {
        "setup.set_starting_plot" => "Choisit l'intrigue de départ.".to_owned(),
        "setup.set_influence_in_shadow_pool" => {
            format!("Place {tokens} dans la réserve des Ténèbres.")
        }
        "setup.set_influence_in_location" => format!("Place {tokens} à {}.", p("location")),
        "draw_cards" | "combat.draw_cards" => {
            let prefix = if entry.action == "draw_cards" {
                ""
            } else {
                "Avant le combat, "
            };
            format!(
                "{prefix}{} pioche {} cartes. Il reste {} cartes dans sa réserve de vie.",
                p("name"),
                p("cards_drawn"),
                p("lp_cards")
            )
        }
        "combat.prepare" => format!(
            "Avant le combat, {} pioche {} cartes et sa force passe de {} à {}.",
            p("name"),
            p("cards_drawn"),
            p("str"),
            p("nstr")
        ),
        "combat.inc_strength" => format!(
            "Avant le combat, {} utilise son agilité pour faire passer sa force de {} à {}.",
            p("name"),
            p("str"),
            p("nstr")
        ),
        "combat.encounter_nothing" => format!(
            "{} fait une rencontre à {}, mais il n'y avait rien.",
            p("hero"),
            p("location")
        ),
        _ => sauron_log_text(entry, data),
    }
}

fn sauron_log_text(entry: &LogEntry, data: &GameData) -> String {
    let p = |name: &str| param(entry, name, data);
    let n = count(entry);
    let plots = plural(n, "carte intrigue", "cartes intrigue");
    let shadows = plural(n, "carte des Ténèbres", "cartes des Ténèbres");
    match entry.action.as_str() {
        "place_influence.success" => format!("Influence fixée à {} à {}.", p("val"), p("location")),
        "place_influence.no_connection" => format!(
            "Aucune liaison avec une forteresse des Ténèbres depuis {}.",
            p("location")
        ),
        "place_influence.max_reached" => format!(
            "Impossible de placer l'influence à {} : elle dépasserait la valeur de la forteresse des Ténèbres liée.",
            p("location")
        ),
        "place_influence.haven" => format!(
            "Impossible de placer de l'influence à {} : c'est un refuge.",
            p("location")
        ),
        "change_shadow_pool" => format!(
            "La réserve des Ténèbres passe de {} à {}.",
            p("old_value"),
            p("new_value")
        ),
        "plot_cards.draw" => format!("Sauron pioche {plots}."),
        "plot_cards.keep" => format!("Sauron garde {plots}."),
        "plot_cards.play" => "Sauron joue une carte intrigue.".to_owned(),
        "plot_cards.discard" => "Sauron retire une carte intrigue du plateau.".to_owned(),
        "plot_cards.discard_gollum" => "L'autre carte Gollum est défaussée.".to_owned(),
        "plot_cards.back_to_bottom" => format!("Sauron replace {plots} sous la pioche."),
        "shadow_cards.draw" => format!("Sauron pioche {shadows}."),
        "shadow_cards.keep" => format!("Sauron garde {shadows}."),
        "shadow_cards.discard" => format!("Sauron défausse {shadows}."),
        "command.move" => format!("Sauron déplace {} vers {}.", p("name"), p("location")),
        "command.place" => format!("Sauron place un monstre à {}.", p("location")),
        "command.heal" => format!("Sauron soigne {}.", p("name")),
        "character.remove" => format!("Retire {}.", p("name")),
        "character.place" => format!("Place {} à {}.", p("name"), p("location")),
        "favor.place" => format!("Place une faveur à {}.", p("location")),
        "event.choose_card" => "Tire une carte événement.".to_owned(),
        "event.resolved" => format!("L'événement « {} » est appliqué.", p("event")),
        "event.activated" => format!("Active l'événement en jeu « {} ».", p("event")),
        "training.keep" => format!(
            "S'entraîne et garde la compétence {}.",
            skill_name(&p("skill"), data)
        ),
        "corruption.discard" => format!("Défausse la carte de corruption {}.", p("corruption")),
        action => {
            if let Some(code) = action.strip_prefix("sauron_actions.place.") {
                format!("Choisit l'action : {}.", sauron_action_name(code))
            } else if let Some(code) = action.strip_prefix("sauron_actions.remove.") {
                format!("Retire le jeton : {}.", sauron_action_name(code))
            } else {
                action.to_owned()
            }
        }
    }
}

/// Picture of the card a log entry refers to, if any.
pub fn log_card_pic(entry: &LogEntry) -> Option<String> {
    let text = |name: &str| {
        entry.params.get(name).map(|v| match v {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        })
    };
    if let Some(card) = text("shadow_card") {
        return Some(image(&format!("sauron/shadows/{card}.png")));
    }
    if let Some(card) = text("plot_card") {
        return Some(image(&format!("sauron/plots/{card}.png")));
    }
    text("event_card").map(|card| image(&format!("events/{card}")))
}
