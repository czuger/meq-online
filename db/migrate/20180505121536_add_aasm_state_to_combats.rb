class AddAasmStateToCombats < ActiveRecord::Migration[5.2]
  def change
    add_column :combats, :aasm_state, :string
  end
end
