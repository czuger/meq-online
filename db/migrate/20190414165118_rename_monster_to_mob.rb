class RenameMonsterToMob < ActiveRecord::Migration[5.2]
  def change
    rename_table :monsters, :mobs

    change_column :mobs, :pool_key, :string, null: true
  end
end
