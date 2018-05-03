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

ActiveRecord::Schema.define(version: 2018_05_03_160410) do

  # These are extensions that must be enabled in order to support this database
  enable_extension "plpgsql"

  create_table "boards", force: :cascade do |t|
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.integer "max_heroes_count", default: 3, null: false
    t.integer "current_heroes_count", default: 0, null: false
    t.boolean "sauron_created", default: false, null: false
    t.string "aasm_state"
  end

  create_table "boards_users", id: false, force: :cascade do |t|
    t.bigint "user_id", null: false
    t.bigint "board_id", null: false
    t.index ["user_id", "board_id"], name: "index_boards_users_on_user_id_and_board_id", unique: true
  end

  create_table "combats", force: :cascade do |t|
    t.bigint "board_id"
    t.bigint "hero_id"
    t.integer "temporary_strength", null: false
    t.string "hero_cards_played", null: false
    t.string "sauron_cards_played", null: false
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.index ["board_id"], name: "index_combats_on_board_id"
    t.index ["hero_id"], name: "index_combats_on_hero_id"
  end

  create_table "heros", force: :cascade do |t|
    t.bigint "board_id"
    t.string "name_code"
    t.integer "fortitude"
    t.integer "strength"
    t.integer "agility"
    t.integer "wisdom"
    t.string "location"
    t.string "hand"
    t.string "life_pool"
    t.string "rest_pool"
    t.string "damage_pool"
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.bigint "user_id", null: false
    t.index ["board_id"], name: "index_heros_on_board_id"
    t.index ["user_id"], name: "index_heros_on_user_id"
  end

  create_table "logs", force: :cascade do |t|
    t.bigint "board_id", null: false
    t.string "action", null: false
    t.string "params", null: false
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.index ["board_id"], name: "index_logs_on_board_id"
  end

  create_table "saurons", force: :cascade do |t|
    t.bigint "board_id", null: false
    t.bigint "user_id", null: false
    t.string "plot_cards", null: false
    t.string "shadow_cards", null: false
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
    t.index ["provider", "uid"], name: "index_users_on_provider_and_uid", unique: true
  end

  add_foreign_key "combats", "boards"
  add_foreign_key "combats", "heros"
  add_foreign_key "heros", "boards"
  add_foreign_key "heros", "users"
  add_foreign_key "logs", "boards"
  add_foreign_key "saurons", "boards"
  add_foreign_key "saurons", "users"
end
