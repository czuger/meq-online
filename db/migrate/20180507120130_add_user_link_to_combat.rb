class AddUserLinkToCombat < ActiveRecord::Migration[5.2]
  def change
    add_column :combats, :sauron_user_id, :bigint, null: false
    add_column :combats, :hero_user_id, :bigint, null: false

    add_foreign_key :combats, :users, column: :sauron_user_id
    add_foreign_key :combats, :users, column: :hero_user_id
  end
end
