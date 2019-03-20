class AddPlayingOrderToActor < ActiveRecord::Migration[5.2]
  def change
    add_column :actors, :playing_order, :integer, limit: 1
  end
end
