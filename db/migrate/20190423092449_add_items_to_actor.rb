class AddItemsToActor < ActiveRecord::Migration[5.2]
  def change
    add_column :actors, :items, :jsonb, null: false, default: {}
  end
end
