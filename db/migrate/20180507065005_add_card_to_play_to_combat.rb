class AddCardToPlayToCombat < ActiveRecord::Migration[5.2]
  def change
    add_column :combats, :sauron_card_to_play, :integer
    add_column :combats, :hero_card_to_play, :integer
  end
end
