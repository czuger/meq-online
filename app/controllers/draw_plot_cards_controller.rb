class DrawPlotCardsController < ApplicationController

  before_action :require_logged_in
  before_action :set_sauron, only: [:edit, :update]

  def edit
    nb_cards = params[:nb_cards].to_i
    @cards = @board.plot_deck.shift(nb_cards)
    @board.log!( current_user, @board.sauron, :draw_plot_cards, { count: @cards.count } )
  end

  def update
    selected_card = params[:selected_card].to_i
    pool_cards = params[:pool_cards].split.map{ |e| e.to_i }

    raise "Selected card should not be 0" if selected_card == 0

    pool_cards.delete(selected_card)
    @sauron.plot_cards << selected_card
    @board.plot_deck += pool_cards

    @board.transaction do
      @board.save!
      @sauron.save!
      @board.log!( current_user, @board.sauron, :keep_plot_card )
    end

    redirect_to board_sauron_path(@board)
  end

  private

  # Use callbacks to share common setup or constraints between actions.
  def set_sauron
    @board = Board.find(params[:board_id])
    @sauron = @board.sauron
  end

end