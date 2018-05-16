class AddCurrentPlotsToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :current_plots, :string, null: false
  end
end
