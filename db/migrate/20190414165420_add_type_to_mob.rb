class AddTypeToMob < ActiveRecord::Migration[5.2]
  def change
    add_column :mobs, :type, :string, null: false
    add_column :mobs, :attack_deck, :string, null: false

    add_column :mobs, :fortitude, :integer, null: false, limit: 1
    add_column :mobs, :strength, :integer, null: false, limit: 1
    add_column :mobs, :life, :integer, null: false, limit: 1
    add_column :mobs, :name, :string, null: false

    add_column :mobs, :hand, :string, null: false
  end
end
