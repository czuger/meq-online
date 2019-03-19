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

ActiveRecord::Schema.define(version: 2019_03_18_193606) do

  # These are extensions that must be enabled in order to support this database
  enable_extension "hstore"
  enable_extension "plpgsql"

  create_table "actors", force: :cascade do |t|
    t.bigint "board_id", null: false
    t.bigint "user_id", null: false
    t.string "type", null: false
    t.string "name_code"
    t.string "name"
    t.string "location"
    t.integer "fortitude", limit: 2
    t.integer "strength", limit: 2
    t.integer "agility", limit: 2
    t.integer "wisdom", limit: 2
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.integer "current_quest"
    t.boolean "turn_finished", default: false, null: false
    t.integer "plot_cards", array: true
    t.integer "shadow_cards", array: true
    t.integer "drawn_plot_cards", array: true
    t.integer "drawn_shadow_cards", array: true
    t.integer "life_pool", array: true
    t.integer "rest_pool", array: true
    t.integer "damage_pool", array: true
    t.integer "hand", array: true
    t.boolean "active", default: false, null: false
    t.index ["board_id"], name: "index_actors_on_board_id"
  end

  create_table "board_messages", force: :cascade do |t|
    t.bigint "sender_id", null: false
    t.bigint "reciever_id", null: false
    t.string "text"
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.index ["reciever_id"], name: "index_board_messages_on_reciever_id"
  end

  create_table "boards", force: :cascade do |t|
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.integer "max_heroes_count", default: 3, null: false
    t.integer "current_heroes_count", default: 0, null: false
    t.boolean "sauron_created", default: false, null: false
    t.string "aasm_state"
    t.integer "plot_deck", null: false, array: true
    t.integer "shadow_deck", null: false, array: true
    t.integer "plot_discard", null: false, array: true
    t.integer "shadow_discard", array: true
    t.integer "shadow_pool", limit: 2, null: false
    t.hstore "influence", null: false
    t.hstore "current_plots", null: false
    t.hstore "characters", null: false
    t.integer "story_marker_heroes", limit: 2, default: 0, null: false
    t.integer "story_marker_ring", limit: 2, default: 0, null: false
    t.integer "story_marker_conquest", limit: 2, default: 0, null: false
    t.integer "story_marker_corruption", limit: 2, default: 0, null: false
    t.hstore "sauron_actions", default: {}, null: false
    t.integer "heroes_objective", limit: 2
    t.integer "sauron_objective", limit: 2
    t.integer "turn", limit: 2, default: 1, null: false
    t.integer "last_event_card", limit: 2
    t.integer "event_deck", limit: 2, null: false, array: true
    t.integer "event_discard", limit: 2, default: [], null: false, array: true
  end

  create_table "boards_users", id: false, force: :cascade do |t|
    t.bigint "user_id", null: false
    t.bigint "board_id", null: false
    t.index ["user_id", "board_id"], name: "index_boards_users_on_user_id_and_board_id", unique: true
  end

  create_table "logs", force: :cascade do |t|
    t.bigint "board_id", null: false
    t.string "action", null: false
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.string "card_pic_path"
    t.bigint "user_id", null: false
    t.bigint "actor_id", null: false
    t.hstore "params", null: false
    t.index ["board_id"], name: "index_logs_on_board_id"
  end

  create_table "movement_preparation_steps", force: :cascade do |t|
    t.bigint "actor_id", null: false
    t.string "origine", null: false
    t.string "destination", null: false
    t.integer "selected_cards", default: [], null: false, array: true
    t.boolean "validation_required", default: true, null: false
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.index ["actor_id"], name: "index_movement_preparation_steps_on_actor_id"
  end

  create_table "users", force: :cascade do |t|
    t.string "provider", null: false
    t.string "uid", null: false
    t.string "name", null: false
    t.string "email", null: false
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.index ["provider", "uid"], name: "index_users_on_provider_and_uid", unique: true
  end

  add_foreign_key "actors", "boards"
  add_foreign_key "actors", "users"
  add_foreign_key "board_messages", "actors", column: "reciever_id"
  add_foreign_key "board_messages", "actors", column: "sender_id"
  add_foreign_key "logs", "actors"
  add_foreign_key "logs", "boards"
  add_foreign_key "logs", "users"
  add_foreign_key "movement_preparation_steps", "actors"
end
