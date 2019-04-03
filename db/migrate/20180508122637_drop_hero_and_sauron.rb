class DropHeroAndSauron < ActiveRecord::Migration[5.2]
  def change
    drop_table :combats
    drop_table :heroes
    drop_table :saurons
  end
end
