class AddObjectivesToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :heroes_objective, :integer, limit: 1
    add_column :boards, :sauron_objective, :integer, limit: 1
  end
end
