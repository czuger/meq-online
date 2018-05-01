class AddMaxPlayersToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :max_heroes_count, :integer, null: false, default: 3
    add_column :boards, :current_heroes_count, :integer, null: false, default: 0
    add_column :boards, :sauron_created, :boolean, null: false, default: false
  end
end
