class CreateBoardMessages < ActiveRecord::Migration[5.2]
  def change
    create_table :board_messages do |t|
      t.references :sender, foreign_key: {to_table: :actors}, index: false
      t.references :reciever, foreign_key: {to_table: :actors}
      t.string :text

      t.timestamps
    end
  end
end