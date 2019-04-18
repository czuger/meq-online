class CombatTableIsBack < ActiveRecord::Migration[5.2]
  def change
    create_table :combats do |t|
      t.references :board, foreign_key: true, null: false, index: { unique: true }
      t.references :actor, foreign_key: true, index: false, null: false
      t.references :mob, foreign_key: true, null: false, index: false

      t.integer :temporary_hero_strength, limit: 1

      t.integer :hero_secret_played_card, limit: 1
      t.integer :mob_secret_played_card, limit: 1

      t.timestamps
    end

    add_column :mobs, :fortitude, :integer, null: false, limit: 1
    add_column :mobs, :strength, :integer, null: false, limit: 1
    add_column :mobs, :life, :integer, null: false, limit: 1
    add_column :mobs, :name, :string, null: false

    add_column :mobs, :hand, :jsonb, null: false, default: []
  end
end
