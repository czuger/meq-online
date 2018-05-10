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
      ensure_sauron
    end

end
