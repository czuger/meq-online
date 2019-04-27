class AddWinnerToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :winner, :string
  end
end
