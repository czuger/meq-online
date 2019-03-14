class AddTurnToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :turn, :integer, null: false, default: 1
  end
end
