class ChangeSauronActionsType < ActiveRecord::Migration[5.2]
  def change
    remove_column :boards, :sauron_actions
    add_column :boards, :sauron_actions, :jsonb, null: false, default: []
  end
end
