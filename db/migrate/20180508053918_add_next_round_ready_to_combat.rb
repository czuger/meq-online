class AddNextRoundReadyToCombat < ActiveRecord::Migration[5.2]
  def change
    add_column :combats, :sauron_next_round_ready, :boolean, null: false, default: false
    add_column :combats, :hero_next_round_ready, :boolean, null: false, default: false
  end
end
