class ChangeActorSerializedItemsToPostgresSpecificFields < ActiveRecord::Migration[5.2]
  def change

    remove_column :actors, :plot_cards, :string
    add_column :actors, :plot_cards, :integer, array: true

    remove_column :actors, :shadow_cards, :string
    add_column :actors, :shadow_cards, :integer, array: true

    remove_column :actors, :drawn_plot_cards, :string
    add_column :actors, :drawn_plot_cards, :integer, array: true

    remove_column :actors, :drawn_shadow_cards, :string
    add_column :actors, :drawn_shadow_cards, :integer, array: true

    remove_column :actors, :life_pool, :string
    add_column :actors, :life_pool, :integer, array: true

    remove_column :actors, :rest_pool, :string
    add_column :actors, :rest_pool, :integer, array: true

    remove_column :actors, :damage_pool, :string
    add_column :actors, :damage_pool, :integer, array: true

    remove_column :actors, :hand, :string
    add_column :actors, :hand, :integer, array: true

    change_column :actors, :fortitude, :integer, limit: 1
    change_column :actors, :strength, :integer, limit: 1
    change_column :actors, :agility, :integer, limit: 1
    change_column :actors, :wisdom, :integer, limit: 1

  end
end
