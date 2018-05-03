class CreateCombats < ActiveRecord::Migration[5.2]
  def change
    create_table :combats do |t|
      t.references :board, foreign_key: true
      t.references :hero, foreign_key: true
      t.integer :temporary_strength, null: false
      t.string :hero_cards_played, null: false
      t.string :sauron_cards_played, null: false

      t.timestamps
    end
  end
end
