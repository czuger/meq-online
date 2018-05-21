class AddCurrentQuestToActor < ActiveRecord::Migration[5.2]
  def change
    add_column :actors, :current_quest, :integer
  end
end
