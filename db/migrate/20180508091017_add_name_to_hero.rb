class AddNameToHero < ActiveRecord::Migration[5.2]
  def change
    add_column :heroes, :name, :string, null: false
  end
end
