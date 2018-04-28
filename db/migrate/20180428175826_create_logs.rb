class CreateLogs < ActiveRecord::Migration[5.2]
  def change
    create_table :logs do |t|
      t.references :board, foreign_key: true, index: true, null: false
      t.string :action, null: false
      t.string :params, null: false

      t.timestamps
    end
  end
end
