-- Database schema, applied at every startup. JSON columns hold serde_json text.

CREATE TABLE IF NOT EXISTS users (
    id         INTEGER PRIMARY KEY,
    provider   TEXT    NOT NULL,              -- 'dev', 'ai' (computer seats), later 'discord'
    uid        TEXT    NOT NULL,
    name       TEXT    NOT NULL,
    email      TEXT    NOT NULL,
    ai_index   INTEGER UNIQUE,                -- set for computer seats
    created_at TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (provider, uid)
);

CREATE TABLE IF NOT EXISTS sessions (
    token      TEXT    PRIMARY KEY,
    user_id    INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS boards (
    id                   INTEGER PRIMARY KEY,
    state                TEXT    NOT NULL,    -- BoardState, named after its GET route
    max_heroes           INTEGER NOT NULL,
    waiting_heroes       TEXT    NOT NULL DEFAULT '[]', -- JSON [hero code], open seats
    turn                 INTEGER NOT NULL,
    current_hero_id      INTEGER REFERENCES heroes (id) ON DELETE SET NULL,
    winner               TEXT    CHECK (winner IN ('heroes', 'sauron', 'draw')),
    story_heroes         INTEGER NOT NULL DEFAULT 0,
    story_ring           INTEGER NOT NULL DEFAULT 0,
    story_conquest       INTEGER NOT NULL DEFAULT 0,
    story_corruption     INTEGER NOT NULL DEFAULT 0,
    shadow_pool          INTEGER NOT NULL,
    influence            TEXT    NOT NULL,    -- JSON {location: count}
    characters           TEXT    NOT NULL,    -- JSON {character: location}
    favors               TEXT    NOT NULL,    -- JSON [location], one entry per token
    plots                TEXT    NOT NULL,    -- JSON {slot: plot card}
    sauron_actions       TEXT    NOT NULL,    -- JSON [action token, e.g. "command_1"]
    sauron_actions_count INTEGER NOT NULL DEFAULT 0,  -- actions taken this turn
    current_action       TEXT,                -- JSON ActionInProgress
    decks                TEXT    NOT NULL,    -- JSON {plot|shadow|event|event_ii|event_iii|corruption|skill: {draw, discard}}
    monster_pools        TEXT    NOT NULL,    -- JSON {color: [mob code]}
    monster_discards     TEXT    NOT NULL DEFAULT '{}', -- JSON {color: [mob code]}
    ringwraiths_return   INTEGER NOT NULL DEFAULT 0,
    influence_budget     TEXT    NOT NULL DEFAULT '{}', -- JSON InfluenceBudget
    last_event           TEXT,                -- JSON EventRef
    current_event        TEXT,                -- JSON EventRef, card in play
    held_event           TEXT,                -- JSON HeldEvent
    pending_choice       TEXT,                -- JSON PendingChoice
    turn_effects         TEXT    NOT NULL DEFAULT '{}', -- JSON TurnEffects
    heroes_objective     INTEGER,
    sauron_objective     INTEGER,
    options              TEXT    NOT NULL DEFAULT '{}', -- JSON GameOptions, optional rules
    created_at           TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at           TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS board_members (
    board_id INTEGER NOT NULL REFERENCES boards (id) ON DELETE CASCADE,
    user_id  INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    PRIMARY KEY (board_id, user_id)
);

CREATE TABLE IF NOT EXISTS heroes (
    id                             INTEGER PRIMARY KEY,
    board_id                       INTEGER NOT NULL REFERENCES boards (id) ON DELETE CASCADE,
    user_id                        INTEGER NOT NULL REFERENCES users (id),
    code                           TEXT    NOT NULL,
    location                       TEXT    NOT NULL,
    fortitude                      INTEGER NOT NULL,
    strength                       INTEGER NOT NULL,
    agility                        INTEGER NOT NULL,
    wisdom                         INTEGER NOT NULL,
    favor                          INTEGER NOT NULL DEFAULT 0,
    quest                          INTEGER,
    advanced_quest                 INTEGER,
    playing_order                  INTEGER NOT NULL,
    turn                           INTEGER NOT NULL DEFAULT 1,
    active                         INTEGER NOT NULL DEFAULT 0,
    turn_finished                  INTEGER NOT NULL DEFAULT 0,
    damages_taken_this_turn        INTEGER NOT NULL DEFAULT 0,
    temporary_damages              INTEGER NOT NULL DEFAULT 0,
    cards_drawn_this_round         INTEGER NOT NULL DEFAULT 0,
    life_pool                      TEXT    NOT NULL,  -- JSON [card]
    hand                           TEXT    NOT NULL,  -- JSON [card]
    rest_pool                      TEXT    NOT NULL,  -- JSON [card]
    damage_pool                    TEXT    NOT NULL,  -- JSON [card]
    items                          TEXT    NOT NULL,  -- JSON {horse, boat}
    levels                         TEXT    NOT NULL DEFAULT '{}', -- JSON {fortitude, strength, agility, wisdom}
    turn_record                    TEXT    NOT NULL,  -- JSON TurnRecord
    corruption_cards               TEXT    NOT NULL,  -- JSON [corruption card]
    UNIQUE (board_id, code)
);
CREATE INDEX IF NOT EXISTS heroes_board ON heroes (board_id);

CREATE TABLE IF NOT EXISTS saurons (
    id           INTEGER PRIMARY KEY,
    board_id     INTEGER NOT NULL UNIQUE REFERENCES boards (id) ON DELETE CASCADE,
    user_id      INTEGER NOT NULL REFERENCES users (id),
    active       INTEGER NOT NULL DEFAULT 0,
    plot_hand    TEXT    NOT NULL,            -- JSON [card]
    shadow_hand  TEXT    NOT NULL,            -- JSON [card]
    drawn_plot   TEXT    NOT NULL,            -- JSON [card]
    drawn_shadow TEXT    NOT NULL             -- JSON [card]
);

CREATE TABLE IF NOT EXISTS mobs (
    id                      INTEGER PRIMARY KEY,
    board_id                INTEGER NOT NULL REFERENCES boards (id) ON DELETE CASCADE,
    kind                    TEXT    NOT NULL CHECK (kind IN ('monster', 'minion')),
    code                    TEXT    NOT NULL,
    name                    TEXT    NOT NULL,
    location                TEXT    NOT NULL,
    pool                    TEXT,             -- monster pool color, monsters only
    fortitude               INTEGER NOT NULL,
    strength                INTEGER NOT NULL,
    life                    INTEGER NOT NULL,
    max_life                INTEGER NOT NULL,
    attack_deck             TEXT    NOT NULL,
    hand                    TEXT    NOT NULL, -- JSON [card]
    damages_taken_this_turn INTEGER NOT NULL DEFAULT 0,
    commanded               INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS mobs_board ON mobs (board_id);

CREATE TABLE IF NOT EXISTS combats (
    board_id           INTEGER PRIMARY KEY REFERENCES boards (id) ON DELETE CASCADE,
    hero_id            INTEGER NOT NULL REFERENCES heroes (id) ON DELETE CASCADE,
    mob_id             INTEGER NOT NULL REFERENCES mobs (id) ON DELETE CASCADE,
    origin             TEXT    NOT NULL,      -- JSON CombatOrigin
    modifiers          TEXT    NOT NULL DEFAULT '{}', -- JSON CombatModifiers
    hero_strength      INTEGER,
    hero_secret_card   INTEGER,
    mob_secret_card    INTEGER,
    hero_strength_used INTEGER NOT NULL DEFAULT 0,
    mob_strength_used  INTEGER NOT NULL DEFAULT 0,
    hero_exhausted     INTEGER NOT NULL DEFAULT 0,
    mob_exhausted      INTEGER NOT NULL DEFAULT 0,
    hero_cards         TEXT    NOT NULL,      -- JSON [PlayedCard], oldest first
    mob_cards          TEXT    NOT NULL       -- JSON [PlayedCard], oldest first
);

CREATE TABLE IF NOT EXISTS logs (
    id         INTEGER PRIMARY KEY,
    board_id   INTEGER NOT NULL REFERENCES boards (id) ON DELETE CASCADE,
    actor_kind TEXT    NOT NULL CHECK (actor_kind IN ('system', 'sauron', 'hero')),
    hero_id    INTEGER REFERENCES heroes (id) ON DELETE SET NULL,
    action     TEXT    NOT NULL,
    params     TEXT    NOT NULL,              -- JSON {name: value}
    created_at TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS logs_board ON logs (board_id);

CREATE TABLE IF NOT EXISTS messages (
    id               INTEGER PRIMARY KEY,
    board_id         INTEGER NOT NULL REFERENCES boards (id) ON DELETE CASCADE,
    sender_kind      TEXT    NOT NULL CHECK (sender_kind IN ('sauron', 'hero')),
    sender_hero_id   INTEGER REFERENCES heroes (id) ON DELETE CASCADE,
    receiver_kind    TEXT    NOT NULL CHECK (receiver_kind IN ('sauron', 'hero')),
    receiver_hero_id INTEGER REFERENCES heroes (id) ON DELETE CASCADE,
    text             TEXT    NOT NULL,
    created_at       TEXT    NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS messages_board ON messages (board_id);
