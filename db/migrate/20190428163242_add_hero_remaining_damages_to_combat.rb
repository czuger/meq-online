class AddHeroRemainingDamagesToCombat < ActiveRecord::Migration[5.2]
  def change
    add_column :actors, :temporary_damages, :integer, limit: 1, null: false, default: 0

    remove_column :combat_card_playeds, :final_attack, :integer
    remove_column :combat_card_playeds, :final_defense, :integer

    add_column :mobs, :max_life, :integer, limit: 1, null: false
  end
end
