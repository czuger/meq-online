class CreateMovementPreparationSteps < ActiveRecord::Migration[5.2]
  def change
    create_table :movement_preparation_steps do |t|

      t.references :actor, foreign_key: true, null: false
      t.string :origine, null: false
      t.string :destination, null: false
      t.integer :card_used, null: false
      t.integer :order, null: false
      t.boolean :validation_required, null: false, default: true

      t.timestamps
    end
  end
end
