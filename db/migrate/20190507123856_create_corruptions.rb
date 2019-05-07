class CreateCorruptions < ActiveRecord::Migration[5.2]
  def change
    create_table :corruptions do |t|
      t.references :board, foreign_key: true, index:false
      t.references :actor, foreign_key: true

      t.integer :card_code, null: false, limit: 1
      t.string :name, null: false

      t.integer :favor_cost, null: false, limit: 1

      t.string :flaw
      t.string :modification

      t.timestamps
    end

    add_index :corruptions, [:board_id, :card_code], unique: true

    add_column :boards, :corruption_deck, :integer, null: false, array: true, default: []
    add_column :boards, :corruption_discard, :integer, null: false, array: true, default: []
  end
end