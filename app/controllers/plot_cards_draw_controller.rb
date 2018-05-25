class PlotCardsDrawController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor, only: [:edit, :update]

  def edit
    @cards = @actor.drawn_plot_cards
  end

  def update
    case params[:button]
    when 'draw'
      draw_plot_cards
    when 'keep'
      keep_plot_cards
    when 'put_back'
      back_to_bottom_of_deck
    else
      raise "Unknown order : #{params[:button]}"
    end

    redirect_to edit_plot_cards_draw_path(@actor)
  end

  private

  def back_to_bottom_of_deck
    cards_count = @actor.drawn_plot_cards.count
    @board.plot_deck += @actor.drawn_plot_cards
    @actor.drawn_plot_cards= []

    @board.transaction do
      @board.save!
      @actor.save!
      @board.log!( current_user, @board.sauron, :cards_bottom_plot_deck, { count: cards_count } )
    end
  end

  def keep_plot_cards
    selected_card = JSON.parse(params[:selected_card]).map{ |e| e.to_i }

    raise "Selected cards should not be empty" if selected_card.empty?
    raise "Selected cards should be included in pool cards" unless (selected_card - @actor.drawn_plot_cards).empty?

    @actor.drawn_plot_cards -= selected_card
    @actor.plot_cards += selected_card

    @board.transaction do
      @board.save!
      @actor.save!
      @board.log!( current_user, @board.sauron, :keep_plot_card, { count: selected_card.count } )
    end
  end

  def draw_plot_cards
    nb_cards = params[:nb_cards].to_i
    @cards = @board.plot_deck.shift(nb_cards)
    @actor.drawn_plot_cards= @cards

    @board.transaction do
      @actor.save!
      @board.save!
      @board.log!( current_user, @board.sauron, :draw_plot_cards, { count: @cards.count } )
    end
  end

end