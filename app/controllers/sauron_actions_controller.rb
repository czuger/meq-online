class SauronActionsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_board

  def edit
  end

  def update
    actions_array = params[:actions].to_a

    ActiveRecord::Base.transaction do

      log_add = actions_array - @board.sauron_actions
      log_add.each do |action|
        @board.log( @actor, 'sauron_actions.place.' + action )
      end

      log_remove = @board.sauron_actions - actions_array
      log_remove.each do |action|
        @board.log( @actor, 'sauron_actions.remove.' + action )
      end

      @board.sauron_actions = actions_array
      @board.sauron_actions_count += 1
      @board.save!

      render json: @board.sauron_actions_count
    end

    # flash[:success] = 'Actions updated successfully.'

    # redirect_to edit_sauron_action_path(@actor)
  end

  def set_influence
    location = params[:location]
    val = params[:val].to_i

    set_influence_result = nil

    if @board.location_exists?( location )
      @board.transaction do

        max_influence_for_location = GameEngine::InfluencePaths.new(@board).max_for_new_token(location)

        if max_influence_for_location == 0
          @board.log( @actor, 'place_influence.no_connection', location: @board.location_name(location) )
          set_influence_result = I18n.t( 'logs.show.place_influence.no_connection', location: @board.location_name(location) )

        elsif max_influence_for_location < val
          @board.log( @actor, 'place_influence.max_reached', location: @board.location_name(location), val: val )
          set_influence_result = I18n.t( 'logs.show.place_influence.max_reached', location: @board.location_name(location), val: val )

        elsif @board.influence[location].to_i != val || !@board.influence[location]
          @board.influence[location] = val

          @board.log( @actor, 'place_influence.success', location: @board.location_name(location), val: val )
        end
        @board.save!
      end
    end

    render json: { result: set_influence_result == nil, message: set_influence_result, location: location, val: val }
  end

  # Called when Sauron terminate his turn
  def terminate
    # Hero cards are not drawn at the setup of the game, but at this step.

    ActiveRecord::Base.transaction do
      @board.set_heroes_activation_state( true )
      @board.set_sauron_activation_state( false )

      @board.next_to_finish_sauron_turn!
      @board.next_to_hero_draw_cards_screen!

      @board.sauron_actions_count = 0
      @board.save!

      RefreshChannel.refresh

      redirect_to boards_path
    end
  end

end
