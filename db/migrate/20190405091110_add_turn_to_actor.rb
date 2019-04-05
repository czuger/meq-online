class AddTurnToActor < ActiveRecord::Migration[5.2]
  def change
    add_column :actors, :turn, :integer, limit: 1, null: false, default: 1
  end
end
