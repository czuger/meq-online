class CombatIndexUnique < ActiveRecord::Migration[5.2]
  def change
    remove_index :combats, :board_id
    remove_index :combats, :hero_id

    add_index :combats, :board_id, unique: true
    add_index :combats, :hero_id, unique: true
  end
end
