# This file is auto-generated from the current state of the database. Instead
# of editing this file, please use the migrations feature of Active Record to
# incrementally modify your database, and then regenerate this schema definition.
#
# Note that this schema.rb definition is the authoritative source for your
# database schema. If you need to create the application database on another
# system, you should be using db:schema:load, not running all the migrations
# from scratch. The latter is a flawed and unsustainable approach (the more migrations
# you'll amass, the slower it'll run and the greater likelihood for issues).
#
# It's strongly recommended that you check this file into your version control system.

ActiveRecord::Schema.define(version: 2020_11_09_174922) do

  create_table "board_messages", force: :cascade do |t|
    t.integer "sender_id", null: false
    t.integer "reciever_id", null: false
    t.string "text"
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.index ["reciever_id"], name: "index_board_messages_on_reciever_id"
  end

  create_table "board_plots", force: :cascade do |t|
    t.integer "board_id", null: false
    t.integer "plot_position", limit: 1, null: false
    t.integer "plot_card", limit: 1, null: false
    t.string "affected_location", null: false
    t.string "story_type", null: false
    t.integer "story_advance", limit: 1, null: false
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.integer "favor_to_discard", limit: 1, null: false
    t.index ["board_id"], name: "index_board_plots_on_board_id"
  end

  create_table "boards", force: :cascade do |t|
    t.string "heroes"
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.integer "max_heroes_count", default: 3, null: false
    t.integer "current_heroes_count", default: 0, null: false
    t.boolean "sauron_created", default: false, null: false
    t.string "aasm_state"
    t.string "influence", null: false
    t.string "plot_deck", null: false
    t.string "shadow_deck", null: false
    t.string "plot_discard", null: false
    t.string "shadow_discard", null: false
    t.string "characters", null: false
    t.integer "shadow_pool", limit: 1, null: false
    t.integer "story_marker_heroes", limit: 1, default: 0, null: false
    t.integer "story_marker_ring", limit: 1, default: 0, null: false
    t.integer "story_marker_conquest", limit: 1, default: 0, null: false
    t.integer "story_marker_corruption", limit: 1, default: 0, null: false
    t.string "sauron_actions", null: false
    t.integer "heroes_objective", limit: 1
    t.integer "sauron_objective", limit: 1
    t.integer "turn", limit: 1, default: 1, null: false
    t.integer "last_event_card", limit: 1
    t.integer "event_deck", limit: 1, null: false
    t.integer "event_discard", limit: 1, null: false
    t.string "favors", null: false
    t.integer "current_hero_id"
    t.string "monsters_pool_orange", null: false
    t.string "monsters_pool_purple", null: false
    t.string "monsters_pool_dark_blue", null: false
    t.string "monsters_pool_brown", null: false
    t.string "monsters_pool_dark_green", null: false
    t.string "winner"
    t.integer "sauron_actions_count", limit: 1, default: 0, null: false
    t.integer "corruption_deck", null: false
    t.integer "corruption_discard", null: false
    t.index ["current_hero_id"], name: "index_boards_on_current_hero_id"
  end

  create_table "boards_users", id: false, force: :cascade do |t|
    t.integer "user_id", null: false
    t.integer "board_id", null: false
    t.index ["user_id", "board_id"], name: "index_boards_users_on_user_id_and_board_id", unique: true
  end

  create_table "combat_card_playeds", force: :cascade do |t|
    t.integer "combat_id"
    t.string "type", null: false
    t.integer "card", limit: 1, null: false
    t.string "pic_path", null: false
    t.string "name", null: false
    t.string "power", null: false
    t.integer "strength_cost", limit: 1, null: false
    t.integer "printed_attack", limit: 1, null: false
    t.integer "printed_defense", limit: 1, null: false
    t.string "card_type", null: false
    t.boolean "cancelled", default: false, null: false
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.index ["combat_id"], name: "index_combat_card_playeds_on_combat_id"
  end

  create_table "combats", force: :cascade do |t|
    t.integer "board_id", null: false
    t.integer "actor_id", null: false
    t.integer "mob_id", null: false
    t.integer "temporary_hero_strength", limit: 1
    t.integer "hero_secret_played_card", limit: 1
    t.integer "mob_secret_played_card", limit: 1
    t.integer "hero_strength_used", limit: 1, default: 0, null: false
    t.integer "mob_strength_used", limit: 1, default: 0, null: false
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.boolean "hero_exhausted", default: false, null: false
    t.boolean "mob_exhausted", default: false, null: false
    t.index ["board_id"], name: "index_combats_on_board_id", unique: true
  end

  create_table "corruptions", force: :cascade do |t|
    t.integer "board_id"
    t.integer "actor_id"
    t.integer "card_code", limit: 1, null: false
    t.string "name", null: false
    t.integer "favor_cost", limit: 1, null: false
    t.string "flaw"
    t.string "modification"
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.index ["actor_id"], name: "index_corruptions_on_actor_id"
    t.index ["board_id", "card_code"], name: "index_corruptions_on_board_id_and_card_code", unique: true
  end

  create_table "heroes", force: :cascade do |t|
    t.integer "board_id", null: false
    t.integer "user_id", null: false
    t.string "type", null: false
    t.string "name_code"
    t.string "name"
    t.string "location"
    t.string "life_pool"
    t.string "hand"
    t.string "rest_pool"
    t.string "damage_pool"
    t.integer "fortitude", limit: 1
    t.integer "strength", limit: 1
    t.integer "agility", limit: 1
    t.integer "wisdom", limit: 1
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.integer "current_quest"
    t.boolean "turn_finished", default: false, null: false
    t.boolean "active", default: false, null: false
    t.integer "playing_order", limit: 1
    t.integer "turn", limit: 1, default: 1, null: false
    t.integer "favor", limit: 1, default: 0
    t.integer "damages_taken_this_turn", limit: 1, default: 0, null: false
    t.string "items"
    t.string "used_powers", null: false
    t.integer "temporary_damages", limit: 1, default: 0, null: false
    t.boolean "corruption_card_discarded_this_turn", default: false, null: false
    t.index ["board_id"], name: "index_heroes_on_board_id"
  end

  create_table "logs", force: :cascade do |t|
    t.integer "board_id", null: false
    t.string "action", null: false
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.string "card_pic_path"
    t.bigint "actor_id"
    t.string "params", null: false
    t.index ["board_id"], name: "index_logs_on_board_id"
  end

  create_table "mobs", force: :cascade do |t|
    t.integer "board_id", null: false
    t.string "pool_key"
    t.string "code", null: false
    t.string "location", null: false
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.string "type", null: false
    t.string "attack_deck", null: false
    t.integer "fortitude", limit: 1, null: false
    t.integer "strength", limit: 1, null: false
    t.integer "life", limit: 1, null: false
    t.string "name", null: false
    t.string "hand", null: false
    t.integer "damages_taken_this_turn", limit: 1, default: 0, null: false
    t.integer "max_life", limit: 1, null: false
    t.index ["board_id"], name: "index_mobs_on_board_id"
  end

  create_table "movement_preparation_steps", force: :cascade do |t|
    t.integer "actor_id", null: false
    t.string "origine", null: false
    t.string "destination", null: false
    t.integer "selected_cards", null: false
    t.boolean "validation_required", default: true, null: false
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.index ["actor_id"], name: "index_movement_preparation_steps_on_actor_id"
  end

  create_table "saurons", force: :cascade do |t|
    t.integer "board_id", null: false
    t.integer "user_id", null: false
    t.string "name_code", null: false
    t.string "name", null: false
    t.string "plot_cards", null: false
    t.string "shadow_cards", null: false
    t.string "drawn_plot_cards", null: false
    t.string "drawn_shadow_cards", null: false
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.index ["board_id"], name: "index_saurons_on_board_id", unique: true
    t.index ["user_id"], name: "index_saurons_on_user_id"
  end

  create_table "users", force: :cascade do |t|
    t.string "provider", null: false
    t.string "uid", null: false
    t.string "name", null: false
    t.string "email", null: false
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.integer "ia_user_id", limit: 1
    t.index ["provider", "uid"], name: "index_users_on_provider_and_uid", unique: true
  end

end
