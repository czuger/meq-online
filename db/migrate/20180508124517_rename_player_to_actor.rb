class RenamePlayerToActor < ActiveRecord::Migration[5.2]
  def change
    rename_table :players, :actors
    rename_column :logs, :player_id, :actor_id
  end
end
