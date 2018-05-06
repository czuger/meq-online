class AddMonsterToCombat < ActiveRecord::Migration[5.2]
  def change
    add_column :combats, :monster, :string, null: false
  end
end
