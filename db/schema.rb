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

ActiveRecord::Schema.define(version: 2018_05_26_113334) do

  # These are extensions that must be enabled in order to support this database
  enable_extension "plpgsql"

  create_table "actors", force: :cascade do |t|
    t.bigint "board_id", null: false
    t.bigint "user_id", null: false
    t.string "type", null: false
    t.string "plot_cards"
    t.string "shadow_cards"
    t.string "name_code"
    t.string "name"
    t.string "location"
    t.string "life_pool"
    t.string "hand"
    t.string "rest_pool"
    t.string "damage_pool"
    t.integer "fortitude"
    t.integer "strength"
    t.integer "agility"
    t.integer "wisdom"
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.string "drawn_plot_cards"
    t.integer "current_quest"
    t.boolean "turn_finished", default: false, null: false
    t.string "drawn_shadow_cards"
    t.index ["board_id"], name: "index_actors_on_board_id"
  end

  create_table "boards", force: :cascade do |t|
    t.datetime "created_at", null: false
    t.datetime "updated_at", null: false
    t.integer "max_heroes_count", default: 3, null: false
    t.integer "current_heroes_count", default: 0, null: false
    t.boolean "sauron_created", default: false, null: false
    t.string "aasm_state"
    t.string "influence", null: false
    t.string "shadow_pool", default: "0", null: false
    t.string "plot_deck", null: false
    t.string "shadow_deck", null: false
    t.string "plot_discard", null: false
    t.string "shadow_discard", null: false
    t.string "current_plots", null: false
    t.string "characters", null: false
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
