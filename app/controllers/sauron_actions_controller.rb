class SauronActionsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_board, only: [:edit, :update]

  def edit
  end

  def update
    actions_hash = {}

    ActiveRecord::Base.transaction do

      %w( place_influence draw_cards command ).each do |action|
        1.upto(3).each do |index|
          action_code = "#{action}_#{index}"

          if params[ action_code ]
            actions_hash[ action_code ] = params[ action_code ]

            @board.logs.create!( action: 'sauron_actions.' + action_code, params: {},
                                 user_id: current_user.id, actor: @actor)
          end
        end
      end

      @board.update!( sauron_actions: actions_hash )

    end

    redirect_to edit_sauron_action_path(@actor)
  end

end
