class CreateBoardPlots < ActiveRecord::Migration[5.2]
  def change
    create_table :board_plots do |t|

      t.references :board, foreign_key: true, null: false

      t.integer :plot_position, limit: 1, null: false
      t.integer :plot_card, limit: 1, null: false
      t.string :affected_location, null: false
      t.string :story_type, null: false
      t.integer :story_advance, limit: 1, null: false

      t.timestamps
    end

    remove_column :boards, :current_plots, :string
  end
end
