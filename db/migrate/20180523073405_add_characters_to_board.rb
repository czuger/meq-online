class AddCharactersToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :characters, :string, null: false
  end
end
