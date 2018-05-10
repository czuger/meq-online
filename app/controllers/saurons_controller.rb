class SauronsController < ApplicationController
  before_action :require_logged_in
  before_action :set_sauron, only: [:show]

  def show
    @player = @sauron
    @nb_cards = @sauron.drawn_plot_cards && !@sauron.drawn_plot_cards.empty? ? nil : 2
  end

  private
    # Use callbacks to share common setup or constraints between actions.
    def set_sauron
      @board = Board.find(params[:board_id])
      @sauron = @board.sauron
    end

    # Never trust parameters from the scary internet, only allow the white list through.
    def sauron_params
      params.require(:sauron).permit(:board_id, :user_id, :plot_cards, :shadow_cards)
    end
end
