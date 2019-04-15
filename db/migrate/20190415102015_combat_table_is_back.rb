class CombatTableIsBack < ActiveRecord::Migration[5.2]
  def change
    create_table :combats do |t|
      t.references :board, foreign_key: true, null: false, index: :unique
      t.references :actor, foreign_key: true, null: false, index: :false
      t.references :mob, foreign_key: true, null: false, index: :false

      t.integer :temporary_hero_strength
      t.jsonb :hero_cards_played, null: false, default: []
      t.jsonb :mob_cards_played, null: false, default: []

      t.timestamps
    end

    add_column :mobs, :fortitude, :integer, null: false
    add_column :mobs, :strength, :integer, null: false
    add_column :mobs, :life, :integer, null: false
    add_column :mobs, :name, :string, null: false

    add_column :mobs, :hand, :jsonb, null: false, default: []
  end
end
