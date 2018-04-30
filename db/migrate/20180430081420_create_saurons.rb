class CreateSaurons < ActiveRecord::Migration[5.2]
  def change
    create_table :saurons do |t|
      t.references :board, foreign_key: true, null: false, index: { unique: true }
      t.references :user, foreign_key: true, null: false
      t.string :plot_cards, null: false
      t.string :shadow_cards, null: false

      t.timestamps
    end
  end
end
