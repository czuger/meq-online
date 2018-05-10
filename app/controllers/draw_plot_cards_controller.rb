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
        @board.log!( current_user, @board.sauron, :keep_plot_card, { count: selected_card.count } )
      end
    else
      cards_count = @sauron.drawn_plot_cards.count
      @board.plot_deck += @sauron.drawn_plot_cards
      @sauron.drawn_plot_cards= []

      @board.transaction do
        @board.save!
        @sauron.save!
        @board.log!( current_user, @board.sauron, :cards_bottom_plot_deck, { count: cards_count } )
      end
    end

    redirect_to @sauron
  end

  private

  # Use callbacks to share common setup or constraints between actions.
  def set_sauron
    @board = Board.find(params[:board_id])
    @sauron = @board.sauron
    ensure_sauron
  end

end