class RemoveUserIdFromLog < ActiveRecord::Migration[5.2]
  def change
    remove_column :logs, :user_id, :bigint
    change_column :logs, :actor_id, :bigint, null: false
  end
end
