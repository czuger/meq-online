class ShadowPoolsController < ApplicationController

  before_action :require_logged_in
  before_action :set_sauron_ensure_board

  def update_from_map
    val = params['current_val'].to_i
    old_value = @board.shadow_pool

    if val == 0
      new_value = old_value + 1
    else
      new_value = old_value - 1
    end

    # We limit the max amount of tokens that could be added  to the shadow pool.
    new_value = [ new_value, @board.story_stage * 4 ].min

    update_shadow_pool old_value, new_value

    render :json => new_value
  end

  private

  def update_shadow_pool( old_value, new_value )
    @board.transaction do
      if old_value != new_value
        @board.log( @actor, :change_shadow_pool, old_value: old_value, new_value: new_value )
        @board.update!(shadow_pool: new_value )
      end
    end
  end

end