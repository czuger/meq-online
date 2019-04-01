class SauronController < ApplicationController
  before_action :require_logged_in
  before_action :set_actor_ensure_actor

  def show
  end

  def setup
    @plot_card = GameData::Plots.new.get(@board.current_plots['plot-card-1'])
  end

  def setup_finished
    @board.transaction do
      GameData::Objectives.set_objectives @board

      GameData::Events.new.draw_next_event_card(@board)

      @board.next_to_event_step!
      @board.next_to_sauron_actions!

      @board.set_sauron_activation_state( true )
    end

    redirect_to edit_sauron_action_path(@actor)
  end

  #
  # Mouvement break schedule methods
  #
  def movement_break_schedule_screen
  end

  def movement_break_schedule_add
  end

  def movement_break_schedule_finished
    @board.transaction do
      @board.next_to_movement!
      @board.activate_current_hero
      @board.save!

      redirect_to boards_path
    end
  end

  #
  # Story methods methods
  #
  def story_screen
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

  def story_update
    ActiveRecord::Base.transaction do

      %w( story_marker_heroes story_marker_ring story_marker_conquest story_marker_corruption ).each do |sm|

        old_value = @board.send( sm  )
        new_value = params[sm].to_i

        @board.update!( sm => new_value )

        @board.logs.create!( action: :update_story_marker,
                             params:{
                                 old_value: old_value, new_value: new_value, story_marker: sm.gsub( 'story_marker_', '' ) },
                             actor: @actor)
      end

      redirect_to sauron_story_screen_path( @actor ), notice: 'Story track successfully updated.'
    end
  end

  def story_step_finished
    @board.transaction do
      @board.next_to_plot!

      redirect_to plot_cards_play_screen_path(@actor)
    end
  end

end
