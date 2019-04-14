class AddTypeToMob < ActiveRecord::Migration[5.2]
  def change
    add_column :mobs, :type, :string, null: false
  end
end
