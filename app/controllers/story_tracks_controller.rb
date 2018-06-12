class StoryTracksController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_board, only: [:edit, :update]

  def edit
    @heroes_to_final = 18 - @board.story_marker_heroes

    sauron_highest_marker = [ @board.story_marker_ring, @board.story_marker_conquest, @board.story_marker_corruption ].max
    @sauron_to_final = 18 - sauron_highest_marker

    shadowfall_points = 0
    %w( story_marker_ring story_marker_conquest story_marker_corruption ).each do |sm|
      shadowfall_points += [ @board.send( sm ), 10 ].min
    end
    @sauron_to_shadowfall = 30 - shadowfall_points

    @sauron_dominance = [ @sauron_to_final, @sauron_to_shadowfall ].min

    if @heroes_to_final < @sauron_dominance
      @dominance = :heroes
    elsif @heroes_to_final > @sauron_dominance
      @dominance = :sauron
    else
      @dominance = :nobody
    end
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

      redirect_to edit_story_track_path( @actor )
    end
  end
end
