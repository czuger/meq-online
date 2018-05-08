class AddPlayerToLog < ActiveRecord::Migration[5.2]
  def change
    add_reference :logs, :player, foreign_key: true, index: false, null: false
  end
end
