class ChangeActorIdToNotNullInLogs < ActiveRecord::Migration[5.2]
  def change
    change_column :logs, :actor_id, :bigint, null: true
  end
end
