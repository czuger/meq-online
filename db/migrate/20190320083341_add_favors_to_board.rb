class AddFavorsToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :favors, :jsonb, null: false, default: []
  end
end
