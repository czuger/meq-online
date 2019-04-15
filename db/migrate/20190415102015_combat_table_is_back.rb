class CombatTableIsBack < ActiveRecord::Migration[5.2]
  def change
    create_table :combats do |t|
      t.references :board, foreign_key: true, null: false, index: :unique
      t.references :actor, foreign_key: true, null: false, index: :false

      t.integer :temporary_hero_strength
      t.jsonb :hero_cards_played, null: false, default: []

      t.integer :mob_fortitude, null: false
      t.integer :mob_strength, null: false
      t.integer :mob_life, null: false
      t.string :mob_name, null: false

      t.jsonb :mob_cards, null: false, default: []
      t.jsonb :mob_cards_played, null: false, default: []

      t.timestamps
    end
  end
end
