class AddMonstersPoolsToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :monsters_pool_orange, :jsonb, null: false, default: []
    add_column :boards, :monsters_pool_purple, :jsonb, null: false, default: []
    add_column :boards, :monsters_pool_dark_blue, :jsonb, null: false, default: []
    add_column :boards, :monsters_pool_brown, :jsonb, null: false, default: []
    add_column :boards, :monsters_pool_dark_green, :jsonb, null: false, default: []
  end
end