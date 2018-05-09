class DrawPlotCardsController < ApplicationController

  before_action :require_logged_in
  before_action :set_sauron, only: [:edit, :update]

  def edit
    @cards = []

    if !@sauron.drawn_plot_cards || @sauron.drawn_plot_cards.empty?
      nb_cards = params[:nb_cards].to_i
      @cards = @board.plot_deck.shift(nb_cards)
      @sauron.drawn_plot_cards= @cards

      @board.transaction do

        @sauron.save!
        @board.save!
        @board.log!( current_user, @board.sauron, :draw_plot_cards, { count: @cards.count } )
      end
    else
      @cards = @sauron.drawn_plot_cards
    end
  end

  def update
    if params[:button] == 'keep'
      selected_card = JSON.parse(params[:selected_card]).map{ |e| e.to_i }

      raise "Selected cards should not be empty" if selected_card.empty?
      raise "Selected cards should be included in pool cards" unless (selected_card - @sauron.drawn_plot_cards).empty?

      @sauron.drawn_plot_cards -= selected_card
      @sauron.plot_cards += selected_card

      @board.transaction do
        @board.save!
        @sauron.save!
        @board.log!( current_user, @board.sauron, :keep_plot_card )
      end
    else
      @board.plot_deck += @sauron.drawn_plot_cards
      @sauron.drawn_plot_cards= []

      @board.transaction do
        @board.save!
        @sauron.save!
      end
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