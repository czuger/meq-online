class AddLastEventCardToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :last_event_card, :integer, limit: 1
    add_column :boards, :event_deck, :integer, limit: 1, array: true, null: false
    add_column :boards, :event_discard, :integer, limit: 1, array: true, null: false, default: []
  end
end
