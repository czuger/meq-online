class ShadowPoolsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_board

  def edit
  end

  def update
    old_value = @board.shadow_pool
    new_value = params[:new_shadow_pool].to_i

    update_shadow_pool old_value, new_value

    redirect_to @actor
  end

  def update_from_map
    val = params['current_val'].to_i
    old_value = @board.shadow_pool

    if val == 0
      new_value = old_value + 1
    else
      new_value = old_value - 1
    end

    update_shadow_pool old_value, new_value

    render :json => new_value
  end

  private

  def update_shadow_pool( old_value, new_value )
    @board.transaction do
      @board.logs.create!( action: :change_shadow_pool, params:{ old_value: old_value, new_value: new_value },
                           user_id: current_user.id, actor: @actor)

      @board.update!(shadow_pool: new_value )
    end
  end

end