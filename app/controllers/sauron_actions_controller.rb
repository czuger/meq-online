class SauronActionsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_board, only: [:edit, :update]

  def edit
  end

  def update
    actions_hash = {}

    %w( place_influence draw_cards command ).each do |action|
      1.upto(3).each do |index|
        actions_hash[ "#{action}_#{index}" ] = params[ "#{action}_#{index}" ] if params[ "#{action}_#{index}" ]
      end
    end

    @board.update!( sauron_actions: actions_hash )

    redirect_to edit_sauron_action_path(@actor)
  end

end
