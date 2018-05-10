class AddShadowDeckToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :shadow_deck, :string, null: false
    add_column :boards, :plot_discard, :string, null: false
    add_column :boards, :shadow_discard, :string, null: false
  end
end
