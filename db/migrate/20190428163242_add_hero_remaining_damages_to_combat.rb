class AddHeroRemainingDamagesToCombat < ActiveRecord::Migration[5.2]
  def change
    add_column :actors, :temporary_damages, :integer, limit: 1, null: false, default: 0
  end
end
