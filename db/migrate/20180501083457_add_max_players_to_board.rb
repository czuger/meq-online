class AddMaxPlayersToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :max_players, :integer, null: false, default: 4
    add_column :boards, :current_players_count, :integer, null: false, default: 0
  end
end
