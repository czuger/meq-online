class AddIaUserId < ActiveRecord::Migration[5.2]
  def change
    add_column :users, :ia_user_id, :integer, null: true, limit: 1
  end
end
