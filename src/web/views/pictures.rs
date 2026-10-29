//! Paths of card and token pictures.

use crate::game_data::GameData;
use crate::game_data::combat_card_def::CardNumber;

/// A card shown on a page.
#[derive(Debug, Clone)]
pub struct CardView {
    /// Card number sent back when the card is selected.
    pub id: u32,
    /// Picture URL.
    pub pic: String,
}

/// URL of a static picture.
pub fn image(path: &str) -> String {
    format!("/static/images/{path}")
}

/// Card of a hero deck or skill.
pub fn hero_card(data: &GameData, hero: &str, card: CardNumber) -> CardView {
    let pic = data.hero_card(hero, card).map_or_else(
        |_| String::new(),
        |c| image(&format!("hero_cards/actions/{}", c.pic_path)),
    );
    CardView { id: card, pic }
}

/// Card of a mob attack deck.
pub fn mob_card(data: &GameData, deck: &str, card: CardNumber) -> CardView {
    let pic = data.mob_card(deck, card).map_or_else(
        |_| String::new(),
        |c| image(&format!("monsters/{}", c.pic_path)),
    );
    CardView { id: card, pic }
}

/// Plot card.
pub fn plot_card(card: u32) -> CardView {
    CardView {
        id: card,
        pic: image(&format!("sauron/plots/{card}.png")),
    }
}

/// Shadow card.
pub fn shadow_card(card: u32) -> CardView {
    CardView {
        id: card,
        pic: image(&format!("sauron/shadows/{card}.png")),
    }
}

/// Corruption card.
pub fn corruption_card(card: u32) -> CardView {
    CardView {
        id: card,
        pic: image(&format!("corruption/{card}.png")),
    }
}

/// Cards of a hero's hand.
pub fn hero_hand(data: &GameData, hero: &str, hand: &[CardNumber]) -> Vec<CardView> {
    hand.iter().map(|c| hero_card(data, hero, *c)).collect()
}
