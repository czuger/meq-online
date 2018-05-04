class AddNotNullToHeroBoardId < ActiveRecord::Migration[5.2]
  def change
    change_column :heros, :board_id, :bigint, :null => false
  end
end
