class StoryTracksController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_board, only: [:edit, :update]

  def edit
  end

  def update
    ActiveRecord::Base.transaction do

      %w( story_marker_heroes story_marker_ring story_marker_conquest story_marker_corruption ).each do |sm|

        old_value = @board.send( sm  )
        new_value = params[sm].to_i

        @board.update!( sm => new_value )

        @board.logs.create!( action: :update_story_marker,
          params:{
              old_value: old_value, new_value: new_value, story_marker: sm.gsub( 'story_marker_', '' ) },
                user_id: current_user.id, actor: @actor)
      end

    end
  end
end
