class AddUsedPowersToActor < ActiveRecord::Migration[5.2]
  def change
    add_column :actors, :used_powers, :jsonb, null: false, default: {}
  end
end
