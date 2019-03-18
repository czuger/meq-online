class AddActiveToActor < ActiveRecord::Migration[5.2]
  def change
    add_column :actors, :active, :boolean, null: false, default: false
  end
end
