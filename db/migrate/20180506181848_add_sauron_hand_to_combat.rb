class AddSauronHandToCombat < ActiveRecord::Migration[5.2]
  def change
    add_column :combats, :sauron_hand, :string, null: false
  end
end
