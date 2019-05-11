class AddCorruptionCardDiscardedThisTurnToActor < ActiveRecord::Migration[5.2]
  def change
    add_column :actors, :corruption_card_discarded_this_turn, :boolean, null: false, default: false
  end
end
