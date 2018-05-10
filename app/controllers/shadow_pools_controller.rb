class ShadowPoolsController < ApplicationController

  before_action :require_logged_in
  before_action :set_board, only: [:edit, :update]

  def edit
  end

  def update
    old_value = @board.shadow_pool
    new_value = params[:new_shadow_pool].to_i

    @board.transaction do
        @board.logs.create!( action: :change_shadow_pool, params:{ old_value: old_value, new_value: new_value },
                             user_id: current_user.id, actor_id: @actor_id)

        @board.update!(shadow_pool: new_value )
    end

    redirect_to board_sauron_path(@board)
  end

  private

  def set_board
    @board = Board.find(params[:board_id])
    @actor_id = params[:actor_id].to_i
    raise "Board #{@board.inspect} can't be modified by #{current_user.id}" unless @board.users.pluck(:id).include?(current_user.id)
  end
end