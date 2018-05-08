class AddShadowPoolToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :shadow_pool, :string, null: false, default: 0
  end
end
