class ChangeLogSerializedItemsToPostgresSpecificFields < ActiveRecord::Migration[5.2]
  def change

    remove_column :logs, :params, :string
    add_column :logs, :params, :string, null: false
    
  end
end
