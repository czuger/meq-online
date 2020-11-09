class ModifyBoardInfluenceType < ActiveRecord::Migration[5.2]
  def change
    # remove_column :boards, :influence
    # add_column :boards, :influence, :jsonb, null: false, default: {}
    add_column :actors, :favor, :integer, limit: 1, default: 0
  end
end
