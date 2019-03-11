class AddObjectivesToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :heroes_objective, :integer
    add_column :boards, :sauron_objective, :integer
  end
end
