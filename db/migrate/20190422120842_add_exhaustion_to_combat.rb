class AddExhaustionToCombat < ActiveRecord::Migration[5.2]
  def change
    add_column :combats, :hero_exhausted, :boolean, null: false, default: false
    add_column :combats, :mob_exhausted, :boolean, null: false, default: false
  end
end
