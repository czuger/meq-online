//! French text of log entries.

use serde_json::Value;

use crate::game_data::GameData;
use crate::model::log_entry::LogEntry;
use crate::model::turn_step::{
    STEP_BEGIN, STEP_SKIPPED, StepSkip, TURN_DRAW, TURN_HERO, TURN_SAURON, TURN_STOPPED, TurnStep,
    TurnStop,
};
use crate::web::views::mission_label::{heroes_mission_name, sauron_mission_name};
use crate::web::views::pictures::{corruption_card, hero_card, image, mob_card};
use crate::web::views::step_label::{skip_text, step_label, stop_text};

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
        "exploration.encounter_character" => format!(
            "Consulte {} à {} et obtient {}.",
            p("character_name"),
            p("location_name"),
            plural(n, "faveur", "faveurs")
        ),
        "exploration.character_ability" => format!(
            "Consulte {} à {} et utilise sa capacité.",
            p("character_name"),
            p("location_name")
        ),
        "exploration.dark_path" => "Choisit la voie des Ténèbres et gagne une faveur.".to_owned(),
        "exploration.plot.remove" => format!("Retire une intrigue à {}.", p("location_name")),
        _ => step_log_text(entry, data),
    }
}

/// Typed parameter of a log entry.
fn typed<T: serde::de::DeserializeOwned>(entry: &LogEntry, name: &str) -> Option<T> {
    serde_json::from_value(entry.params.get(name)?.clone()).ok()
}

/// Turns and steps, and what the hero's steps did.
fn step_log_text(entry: &LogEntry, data: &GameData) -> String {
    let p = |name: &str| param(entry, name, data);
    let skip = typed::<StepSkip>(entry, "reason").map_or("", skip_text);
    match entry.action.as_str() {
        TURN_SAURON => "Tour de Sauron".to_owned(),
        TURN_DRAW => "Pioche des héros".to_owned(),
        TURN_HERO => {
            let name = p("hero");
            let elided = name.starts_with(['A', 'E', 'I', 'O', 'U', 'É']);
            format!("Tour {}{name}", if elided { "d'" } else { "de " })
        }
        "turn.end" => "Fin du tour.".to_owned(),
        TURN_STOPPED => typed::<TurnStop>(entry, "reason")
            .map_or("Tour interrompu.", stop_text)
            .to_owned(),
        STEP_BEGIN => format!(
            "Étape : {}.",
            typed::<TurnStep>(entry, "step").map_or("?", step_label)
        ),
        STEP_SKIPPED => format!(
            "{} : {skip}",
            typed::<TurnStep>(entry, "step").map_or("?", step_label)
        ),
        "story.stage" => format!("L'histoire passe à l'étape {}.", p("stage")),
        "story.spawn" => format!("{} entre en jeu à {}.", p("name"), p("location")),
        "draw_cards.none" => format!("{} ne pioche aucune carte.", p("name")),
        "draw_cards.blocked" => format!("{} ne peut pas piocher : {skip}", p("name")),
        "rest.rest" | "rest.heal" => {
            let verb = if entry.action == "rest.heal" {
                "Se soigne"
            } else {
                "Se repose"
            };
            format!(
                "{verb} : {} retrouvée(s) dans la réserve de vie, qui en compte {}.",
                plural(count(entry), "carte", "cartes"),
                p("lp_cards")
            )
        }
        "rest.skip" => "Ne se repose pas.".to_owned(),
        "movement.travel" => format!("Se déplace : {} → {}.", p("from"), p("to")),
        "movement.none" => "Ne se déplace pas.".to_owned(),
        "movement.safe" => format!("Aucun monstre, sbire ni péril à {}.", p("location")),
        "plot_cards.pass" => "Sauron ne joue pas de carte intrigue.".to_owned(),
        _ => combat_result_log_text(entry, data),
    }
}

/// Rounds and outcome of a combat.
fn combat_result_log_text(entry: &LogEntry, data: &GameData) -> String {
    let p = |name: &str| param(entry, name, data);
    match entry.action.as_str() {
        "combat.start" => format!("{} affronte {} à {}.", p("hero"), p("mob"), p("location")),
        "combat.round" => format!(
            "Manche : {} subit {} dégât(s), {} subit {} dégât(s) et a encore {} point(s) de vie.",
            p("hero"),
            p("hero_damage"),
            p("mob"),
            p("mob_damage"),
            p("mob_life")
        ),
        "combat.end" => match p("outcome").as_str() {
            "mob_defeated" => format!("{} a vaincu {}.", p("hero"), p("mob")),
            "hero_defeated" => format!("{} est vaincu par {}.", p("hero"), p("mob")),
            _ => format!(
                "{} et {} sont épuisés : personne n'est vaincu.",
                p("hero"),
                p("mob")
            ),
        },
        _ => finale_log_text(entry, data),
    }
}

fn finale_log_text(entry: &LogEntry, data: &GameData) -> String {
    let p = |name: &str| param(entry, name, data);
    let mission = |name: Option<&str>| {
        let outcome = match entry.params.get("fulfilled").and_then(Value::as_bool) {
            Some(true) => "remplie",
            _ => "non remplie",
        };
        name.map_or_else(
            || "Aucune mission.".to_owned(),
            |name| format!("« {name} » : {outcome}."),
        )
    };
    let code = entry.params.get("mission").cloned().unwrap_or(Value::Null);
    match entry.action.as_str() {
        "finale.start" => "La Finale commence.".to_owned(),
        "finale.sauron_mission" => {
            let name = serde_json::from_value(code).ok().map(sauron_mission_name);
            format!("Sauron révèle sa mission {}", mission(name))
        }
        "finale.heroes_mission" => {
            let name = serde_json::from_value(code).ok().map(heroes_mission_name);
            format!("Les héros révèlent leur mission {}", mission(name))
        }
        "finale.advance" if p("side") == "sauron" => {
            "Mission remplie : chaque marqueur de Sauron avance d'une case.".to_owned()
        }
        "finale.advance" => "Mission remplie : le marqueur des héros avance d'une case.".to_owned(),
        "finale.ringwraiths" => format!(
            "Les Nazgûl attendent le champion avec {} points de vie et une endurance de {}.",
            p("life"),
            p("fortitude")
        ),
        "finale.champion" => format!("{} est le champion des héros face aux Nazgûl.", p("hero")),
        "finale.winner" if p("winner") == "heroes" => "Les héros gagnent la partie !".to_owned(),
        "finale.winner" => "Sauron gagne la partie !".to_owned(),
        _ => shadow_log_text(entry, data),
    }
}

fn shadow_log_text(entry: &LogEntry, data: &GameData) -> String {
    let p = |name: &str| param(entry, name, data);
    let tokens = plural(count(entry), "jeton d'influence", "jetons d'influence");
    match entry.action.as_str() {
        "shadow.play" => format!("Sauron joue la carte des Ténèbres « {} ».", p("shadow")),
        "shadow.moved" => format!("Est déplacé vers {} par Sauron.", p("location")),
        "shadow.mordor_influence" => format!("Sauron place {tokens} sur chaque lieu du Mordor."),
        "shadow.take_card" => "Sauron reprend cette carte de la défausse.".to_owned(),
        "shadow.minion" => format!("Sauron place {} à {}.", p("name"), p("location")),
        "shadow.redistribute" => "Sauron répartit à nouveau les cartes de corruption.".to_owned(),
        "shadow.grievous_loss" => format!(
            "Perte cruelle : Sauron peut placer {tokens}, le marqueur des héros avancera d'une case de moins."
        ),
        "corruption.greedy" => "Avide : Sauron pioche une carte des Ténèbres.".to_owned(),
        "plot_cards.location" => format!("L'intrigue affecte {}.", p("location")),
        _ => peril_log_text(entry, data),
    }
}

fn peril_log_text(entry: &LogEntry, data: &GameData) -> String {
    let p = |name: &str| param(entry, name, data);
    let n = count(entry);
    let tokens = plural(n, "jeton d'influence", "jetons d'influence");
    match entry.action.as_str() {
        "peril.draw" => format!(
            "Sauron pioche {} à {}.",
            plural(n, "carte Péril", "cartes Péril"),
            p("location")
        ),
        "peril.none_matching" => format!(
            "Aucune carte Péril n'affecte ce lieu : elles sont défaussées sans effet.{}",
            discarded_perils(entry)
        ),
        "peril.resolve" => format!(
            "Subit le péril « {} ». {}{}",
            p("peril"),
            p("text"),
            discarded_perils(entry)
        ),
        "peril.influence" => format!("Sauron place une influence à {}.", p("location")),
        "damage.dealt" => format!("Subit {}.", plural(n, "dégât", "dégâts")),
        "corruption.gain" => format!("Reçoit la carte de corruption {}.", p("corruption")),
        "corruption.wretched" => format!("Misérable : est envoyé à {}.", p("location")),
        "corruption.favor_limit" => {
            "N'a pas pu prendre toutes les faveurs à cause d'une carte de corruption.".to_owned()
        }
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

/// Names of the Peril cards discarded with no effect, as the end of a sentence.
fn discarded_perils(entry: &LogEntry) -> String {
    let names = typed::<Vec<String>>(entry, "others").unwrap_or_default();
    if names.is_empty() {
        String::new()
    } else {
        format!(" Cartes défaussées : {}.", names.join(", "))
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
        "command.move" => {
            let name = Some(p("name"))
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| "un monstre".to_owned());
            format!("Sauron déplace {name} vers {}.", p("location"))
        }
        "command.place" => format!("Sauron place un monstre à {}.", p("location")),
        "command.heal" => format!("Sauron soigne {}.", p("name")),
        "character.remove" => format!("Retire {}.", p("name")),
        "character.place" => format!("Place {} à {}.", p("name"), p("location")),
        "favor.place" => format!("Place une faveur à {}.", p("location")),
        "event.drawn" => format!(
            "Un camp est dominant : {} tirées, une seule est appliquée.",
            plural(n, "carte événement", "cartes événement")
        ),
        "event.choose_card" if p("event").is_empty() => "Tire une carte événement.".to_owned(),
        "event.choose_card" => format!("Carte événement : « {} ».", p("event")),
        "event.resolved" => format!("L'événement « {} » est appliqué.", p("event")),
        "event.activated" => format!("Active l'événement en jeu « {} ».", p("event")),
        // The Skill card kept goes to the hand: it stays secret.
        "training.keep" => "S'entraîne et garde une compétence.".to_owned(),
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

/// Pictures of the public cards a log entry refers to.
pub fn log_card_pics(entry: &LogEntry, data: &GameData) -> Vec<String> {
    let number = |name: &str| typed::<u32>(entry, name);
    let code = |name: &str| typed::<String>(entry, name).unwrap_or_default();
    let mut pics = Vec::new();
    pics.extend(number("shadow_card").map(|card| image(&format!("sauron/shadows/{card}.png"))));
    pics.extend(number("plot_card").map(|card| image(&format!("sauron/plots/{card}.png"))));
    pics.extend(number("corruption_card").map(|card| corruption_card(card).pic));
    pics.extend(typed::<String>(entry, "event_card").map(|c| image(&format!("events/{c}"))));
    let events = typed::<Vec<String>>(entry, "event_cards").unwrap_or_default();
    pics.extend(events.iter().map(|c| image(&format!("events/{c}"))));
    let hero = code("hero");
    let paid = typed::<Vec<u32>>(entry, "hero_cards").unwrap_or_default();
    pics.extend(paid.iter().map(|c| hero_card(data, &hero, *c).pic));
    pics.extend(number("hero_card").map(|c| hero_card(data, &hero, c).pic));
    pics.extend(number("mob_card").map(|c| mob_card(data, &code("mob_deck"), c).pic));
    pics.retain(|pic| !pic.is_empty());
    pics
}
