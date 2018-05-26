class AddDrawnShadowCardsToActor < ActiveRecord::Migration[5.2]
  def change
    add_column :actors, :drawn_shadow_cards, :string
  end
end
