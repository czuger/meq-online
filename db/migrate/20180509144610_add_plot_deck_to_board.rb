class AddPlotDeckToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :plot_deck, :string, null: false
  end
end
