class AddNotNullToHeroBoardId < ActiveRecord::Migration[5.2]
  def change
    change_column :heroes, :board_id, :bigint, :null => false
  end
end
