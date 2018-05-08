class AddNameToHero < ActiveRecord::Migration[5.2]
  def change
    add_column :heros, :name, :string, null: false
  end
end
