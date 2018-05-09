class AddDrawnPlotCardsToActor < ActiveRecord::Migration[5.2]
  def change
    add_column :actors, :drawn_plot_cards, :string
  end
end
