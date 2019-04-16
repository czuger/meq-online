class AddPlayedCardsToCombat < ActiveRecord::Migration[5.2]
  def change

    remove_column :combats, :mob_cards_played
    remove_column :combats, :hero_cards_played

    add_column :actors, :combat_cards_played, :jsonb, null: false, default: []
    add_column :mobs, :combat_cards_played, :jsonb, null: false, default: []

    add_column :actors, :combat_card_played, :integer
    add_column :mobs, :combat_card_played, :integer

    remove_column :combats, :temporary_hero_strength
    add_column :actors, :combat_temporary_strength, :integer
  end
end
