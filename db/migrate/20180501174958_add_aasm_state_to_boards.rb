class AddAasmStateToBoards < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :aasm_state, :string
  end
end
