class SauronActionsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_board

  def edit
  end

  def show
  end

  def update
    actions_hash = {}

    log_add = []
    log_remove = []

    ActiveRecord::Base.transaction do

      # %w( place_influence draw_cards command ).each do |action|
      #   1.upto(3).each do |index|
      #     action_code = "#{action}_#{index}"
      #
      #     if params[ action_code ]
      #
      #       actions_hash[ action_code ] = params[ action_code ]
      #       log_add << action_code unless @board.sauron_actions[action_code]
      #
      #     else
      #       log_remove << action_code if @board.sauron_actions[action_code]
      #     end
      #   end
      # end
      #
      # log_add.each do |action|
      #   @board.log( @actor, 'sauron_actions.place.' + action )
      # end
      #
      # log_remove.each do |action|
      #   @board.log( @actor, 'sauron_actions.remove.' + action )
      # end

      p params[:actions]

      @board.update!( sauron_actions: params[:actions].to_a )
    end

    flash[:success] = 'Actions updated successfully.'

    # redirect_to edit_sauron_action_path(@actor)
  end

  # Called when Sauron terminate his turn
  def terminate
    # Hero cards are not drawn at the setup of the game, but at this step.

    ActiveRecord::Base.transaction do
      @board.set_heroes_activation_state( true )
      @board.set_sauron_activation_state( false )

      @board.next_to_heroes_draw_cards!

      redirect_to boards_path
    end
  end

end
