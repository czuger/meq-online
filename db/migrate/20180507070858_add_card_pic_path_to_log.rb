class AddCardPicPathToLog < ActiveRecord::Migration[5.2]
  def change
    add_column :logs, :card_pic_path, :string
  end
end
