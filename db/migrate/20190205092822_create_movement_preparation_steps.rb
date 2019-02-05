class CreateMovementPreparationSteps < ActiveRecord::Migration[5.2]
  def change
    create_table :movement_preparation_steps do |t|
      t.references :board, foreign_key: true
      t.references :hero, foreign_key: true
      t.string :origine
      t.string :destination
      t.integer :card_used
      t.integer :order
      t.boolean :validation_required

      t.timestamps
    end
  end
end
