class SauronActionsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_board, only: [:edit, :update]

  def edit
  end

  def update
    actions_hash = {}

    log_add = []
    log_remove = []

    ActiveRecord::Base.transaction do

      %w( place_influence draw_cards command ).each do |action|
        1.upto(3).each do |index|
          action_code = "#{action}_#{index}"

          if params[ action_code ]

            actions_hash[ action_code ] = params[ action_code ]
            log_add << action_code unless @board.sauron_actions[action_code]

          else
            log_remove << action_code if @board.sauron_actions[action_code]
          end
        end
      end

      log_add.each do |action|
        @board.logs.create!( action: 'sauron_actions.place.' + action, params: {}, user_id: current_user.id, actor: @actor)
      end

      log_remove.each do |action|
        @board.logs.create!( action: 'sauron_actions.remove.' + action, params: {}, user_id: current_user.id, actor: @actor)
      end

      @board.update!( sauron_actions: actions_hash )

    end

    flash[:success] = 'Actions updated successfully.'

    redirect_to edit_sauron_action_path(@actor)
  end

end
