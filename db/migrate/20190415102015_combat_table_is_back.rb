class CombatTableIsBack < ActiveRecord::Migration[5.2]
  def change
    create_table :combats do |t|
      t.references :board, foreign_key: true, null: false, index: { unique: true }
      t.references :actor, foreign_key: true, index: false, null: false
      t.references :mob, foreign_key: true, null: false, index: false

      t.integer :temporary_hero_strength, limit: 1

      t.integer :hero_secret_played_card, limit: 1
      t.integer :mob_secret_played_card, limit: 1

      t.integer :hero_strength_used, limit: 1, null: false, default: 0
      t.integer :mob_strength_used, limit: 1, null: false, default: 0

      t.timestamps
    end

    add_column :actors, :damages_taken_this_turn, :integer, limit: 1, null: false, default: 0
    add_column :mobs, :damages_taken_this_turn, :integer, limit: 1, null: false, default: 0

  end
end
