class AddUserToHero < ActiveRecord::Migration[5.2]
  def change
    add_reference :heroes, :user, foreign_key: true, null: false
  end
end
