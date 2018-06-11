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

ActiveRecord::Schema.define(version: 2018_06_11_211058) do

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
    t.index ["board_id"], name: "index_actors_on_board_id"
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
  end

  create_table "boards_users", id: false, force: :cascade do |t|
    t.bigint "user_id", null: false
    t.bigint "board_id", null: false
    t.index ["user_id", "board_id"], name: "index_boards_users_on_user_id_and_board_id", unique: true
  end

  create_table "logs", force: :cascade do |t|
    t.bigint "board_id", null: false
    t.string "action", null: false
    t.string "params", null: false
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.string "card_pic_path"
    t.bigint "user_id", null: false
    t.bigint "actor_id", null: false
    t.index ["board_id"], name: "index_logs_on_board_id"
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
  add_foreign_key "logs", "actors"
  add_foreign_key "logs", "boards"
  add_foreign_key "logs", "users"
end
