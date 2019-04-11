class AddFavorToDiscardToBoardPlot < ActiveRecord::Migration[5.2]
  def change
    add_column :board_plots, :favor_to_discard, :integer, limit: 1, null: false
  end
end
