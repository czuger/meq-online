class PlotCardsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor

  DECK_NAME = 'plot'.freeze

  def play_screen
    @plot_cards = @board.current_plots
  end

  def draw_screen
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

  def draw
    GameEngine::Deck.new(current_user, @board, @actor, DECK_NAME ).draw_cards(params[:nb_cards])
    redirect_to plot_cards_draw_screen_path(@actor)
  end

  def keep
    GameEngine::Deck.new(current_user, @board, @actor, DECK_NAME ).keep_cards(
        params[:selected_cards].split(',').map{ |e| e.to_i } )
    redirect_to plot_cards_draw_screen_path(@actor)
  end

end