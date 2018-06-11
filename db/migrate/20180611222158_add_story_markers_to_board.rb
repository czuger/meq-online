class AddStoryMarkersToBoard < ActiveRecord::Migration[5.2]
  def change
    add_column :boards, :story_marker_heroes, :integer, limit: 1, null: false, default: 0
    add_column :boards, :story_marker_ring, :integer, limit: 1, null: false, default: 0
    add_column :boards, :story_marker_conquest, :integer, limit: 1, null: false, default: 0
    add_column :boards, :story_marker_corruption, :integer, limit: 1, null: false, default: 0
  end
end
