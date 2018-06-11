class ChangeBoardSerializedArraysToRealPostgresArrays < ActiveRecord::Migration[5.2]
  def change

    enable_extension('hstore') unless extension_enabled?('hstore')

    remove_column :boards, :plot_deck, :string
    add_column :boards, :plot_deck, :integer, array: true, null: false

    remove_column :boards, :shadow_deck, :string
    add_column :boards, :shadow_deck, :integer, array: true, null: false

    remove_column :boards, :plot_discard, :string
    add_column :boards, :plot_discard, :integer, array: true, null: false

    remove_column :boards, :shadow_discard, :string
    add_column :boards, :shadow_discard, :integer, array: true

    remove_column :boards, :shadow_pool, :string
    add_column :boards, :shadow_pool, :integer, limit: 1, null: false

    remove_column :boards, :influence, :string
    add_column :boards, :influence, :hstore, null: false

    remove_column :boards, :current_plots, :string
    add_column :boards, :current_plots, :hstore, null: false

    remove_column :boards, :characters, :string
    add_column :boards, :characters, :hstore, null: false
  end
end
