class AddMonstersOnBoardToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :monsters_on_board, :string, null: false
  end
end
