class AddSauronActionsToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :sauron_actions, :string, null: false
  end
end
