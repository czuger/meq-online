class CreateSaurons < ActiveRecord::Migration[5.2]
  def change
    create_table :saurons do |t|
      t.references :board, foreign_key: true, null: false, index: { unique: true }
      t.references :user, foreign_key: true, null: false
      t.string :name, null: false
      t.string :plot_cards, null: false
      t.string :shadow_cards, null: false
      t.string :drawn_plot_cards, null: false
      t.string :drawn_shadow_cards, null: false

      t.timestamps
    end

    remove_columns :actors, :plot_cards
    remove_columns :actors, :shadow_cards
    remove_columns :actors, :drawn_plot_cards
    remove_columns :actors, :drawn_shadow_cards
    remove_columns :actors, :type

    rename_table :actors, :heroes

    rename_column :combats, :actor_id, :hero_id
  end
end
