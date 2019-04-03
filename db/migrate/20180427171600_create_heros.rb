class CreateHeros < ActiveRecord::Migration[5.2]
  def change
    create_table :heroes do |t|
      t.references :board, index: true, foreign_key: true
      t.string :name_code
      t.integer :fortitude
      t.integer :strength
      t.integer :agility
      t.integer :wisdom
      t.string :location
      t.string :hand
      t.string :life_pool
      t.string :rest_pool
      t.string :damage_pool

      t.timestamps
    end
    remove_column :boards, :heroes, :string
  end
end
