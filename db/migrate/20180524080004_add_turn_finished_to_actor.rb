class AddTurnFinishedToActor < ActiveRecord::Migration[5.2]
  def change
    add_column :actors, :turn_finished, :boolean, null: false, default: false
  end
end
