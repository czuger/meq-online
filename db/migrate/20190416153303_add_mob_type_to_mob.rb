class AddMobTypeToMob < ActiveRecord::Migration[5.2]
  def change
    add_column :mobs, :attack_deck, :string, null: false
  end
end
