class AddUserToHero < ActiveRecord::Migration[5.2]
  def change
    add_reference :heros, :user, foreign_key: true, null: true
  end
end
