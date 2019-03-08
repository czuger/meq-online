class PlotCardsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor
  before_action :set_selected_card, only: [ :play, :discard ]

  DECK_NAME = 'plot'.freeze

  def play_screen
    @plot_cards = @board.current_plots
    @free_slots = 1.upto(3).map{ |i| "plot-card-#{i}" } - @board.current_plots.keys

    @free_slots_options = @free_slots.map{ |e| [ e.gsub( 'plot-card-'.freeze, 'Card slot '.freeze ), e ] }.sort
  end

  def play
    @board.current_plots[params[:card_slot]] = @selected_card
    @actor.plot_cards.delete(@selected_card)

    @board.transaction do
      @actor.save!
      @board.save!
      @board.log!( current_user, @board.sauron, 'plot_cards.play', { plot_card: @selected_card } )
    end

    redirect_to plot_cards_play_screen_path(@actor)
  end

  def discard_screen
    @used_slots = @board.current_plots.keys
    @used_slots_options = @used_slots.map{ |e| [ e.gsub( 'plot-card-'.freeze, 'Card slot '.freeze ), e ] }.sort
  end

  def discard
    @board.plot_deck << @card
    @board.current_plots.delete(params[:card_slot])

    @board.transaction do
      @board.save!
      @board.log!( current_user, @board.sauron, 'plot_cards.discard', { plot_card: card } )
    end

    redirect_to plot_cards_discard_screen_path(@actor)
  end

  def draw_screen
    @cards = @actor.drawn_plot_cards
  end

  def draw
    GameEngine::Deck.new(current_user, @board, @actor, DECK_NAME ).draw_cards(params[:nb_cards])
    redirect_to plot_cards_draw_screen_path(@actor)
  end

  def keep
    GameEngine::Deck.new(current_user, @board, @actor, DECK_NAME ).keep_cards(
        params[:selected_cards].split(',').map{ |e| e.to_i } )
    redirect_to plot_cards_draw_screen_path(@actor)
  end

  private

  def set_selected_card
    raise "Bad formatted card slot = #{params[:card_slot]}" unless params[:card_slot] =~ /plot-card-\d/
    @selected_card = params[:selected_card].to_i
  end

end