class AddCurrentHeroToBoard < ActiveRecord::Migration[5.2]
  def change
    add_reference :boards, :current_hero, foreign_key: {to_table: :actors}
  end
end
