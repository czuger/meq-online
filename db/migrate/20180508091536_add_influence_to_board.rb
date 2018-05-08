class AddInfluenceToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :influence, :string, null: false
  end
end
