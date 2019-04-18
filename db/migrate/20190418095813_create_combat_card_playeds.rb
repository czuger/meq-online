class CreateCombatCardPlayeds < ActiveRecord::Migration[5.2]
  def change
    create_table :combat_card_playeds do |t|
      t.references :combat, foreign_key: true
      t.string :type, null: false
      t.integer :card, null: false
      t.string :pic_path, null: false
      t.string :name, null: false
      t.string :power, null: false
      t.integer :strength_cost, null: false
      t.integer :printed_attack, null: false
      t.integer :final_attack, null: false
      t.integer :printed_defense, null: false
      t.integer :final_defense, null: false
      t.boolean :cancelled, null: false, default: false

      t.timestamps
    end
  end
end
