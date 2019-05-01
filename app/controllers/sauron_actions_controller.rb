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

      @board.update!( sauron_actions: actions_array )
    end

    # flash[:success] = 'Actions updated successfully.'

    # redirect_to edit_sauron_action_path(@actor)
  end

  def set_influence
    ActionController::Parameters.permit_all_parameters = true
    @locations= GameData::Locations.new

    tmp_hash = params[:locations].select{ |l, v| !v.empty? && @locations.exist?( l ) }.to_h
    tmp_hash.transform_values!{ |v| v.to_i }
    tmp_hash = tmp_hash

    @board.transaction do
      diff_hash = Hash[ tmp_hash.to_a.sort - @board.influence.to_a.sort ]
      diff_hash.each do |k, v|
        @board.log( @actor, :place_influence, place: @locations.get(k).name, value: v )
      end

      @board.influence.merge!( tmp_hash )
      @board.save!
    end
  end


  # Called when Sauron terminate his turn
  def terminate
    # Hero cards are not drawn at the setup of the game, but at this step.

    ActiveRecord::Base.transaction do
      @board.set_heroes_activation_state( true )
      @board.set_sauron_activation_state( false )

      @board.next_to_finish_sauron_turn!
      @board.next_to_hero_draw_cards_screen!

      redirect_to boards_path
    end
  end

  def heal_minion_screen

  end

  def heal_minion

  end

end
