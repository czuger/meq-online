class CreatePlayers < ActiveRecord::Migration[5.2]
  def change
    create_table :players do |t|

      t.references :board, foreign_key: true, null: false
      t.references :user, foreign_key: true, null: false, index: false
      t.string :type, null: false

      t.string :plot_cards
      t.string :shadow_cards

      t.string :name_code
      t.string :name

      t.string :location
      t.string :life_pool
      t.string :hand
      t.string :rest_pool
      t.string :damage_pool
      t.integer :fortitude
      t.integer :strength
      t.integer :agility
      t.integer :wisdom

      t.timestamps
    end
  end
end
