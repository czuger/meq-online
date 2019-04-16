class SauronActionsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_board

  def edit
  end

  def show
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

  # Called when Sauron terminate his turn
  def terminate
    # Hero cards are not drawn at the setup of the game, but at this step.

    ActiveRecord::Base.transaction do
      @board.set_heroes_activation_state( true )
      @board.set_sauron_activation_state( false )

      @board.next_to_hero_draw_cards!

      redirect_to boards_path
    end
  end

end
