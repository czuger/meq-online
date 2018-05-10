class ShadowPoolsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_board, only: [:edit, :update]

  def edit
  end

  def update
    old_value = @board.shadow_pool
    new_value = params[:new_shadow_pool].to_i

    @board.transaction do
        @board.logs.create!( action: :change_shadow_pool, params:{ old_value: old_value, new_value: new_value },
                             user_id: current_user.id, actor: @actor)

        @board.update!(shadow_pool: new_value )
    end

    redirect_to @actor
  end

end