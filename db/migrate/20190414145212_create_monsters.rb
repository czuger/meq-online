class CreateMonsters < ActiveRecord::Migration[5.2]
  def change
    create_table :monsters do |t|
      t.references :board, foreign_key: true, null: false
      t.string :pool_key, null: false
      t.string :code, null: false
      t.string :location, null: false

      t.timestamps
    end

    remove_column :boards, :monsters_on_board
  end
end
