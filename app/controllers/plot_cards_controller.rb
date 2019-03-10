class PlotCardsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor

  DECK_NAME = 'plot'.freeze

  def play_screen
    @plot_cards = @actor.plot_cards
    @free_slots = 1.upto(3).map{ |i| "plot-card-#{i}" } - @board.current_plots.keys

    @free_slots_options = @free_slots.map{ |e| [ e.gsub( 'plot-card-'.freeze, 'Card slot '.freeze ), e ] }.sort
  end

  def play
    @selected_card_slot = params[:card_slot]
    validate_card_slot_syntax
    selected_card = params[:selected_card].to_i

    raise "Actor does not have card #{selected_card} in #{@actor.plot_cards}" unless @actor.plot_cards.include?(selected_card)
    raise "Slot already in use : #{@selected_card_slot} = #{@board.current_plots[@selected_card_slot]}" if @board.current_plots[@selected_card_slot]

    @board.current_plots[@selected_card_slot] = selected_card
    @actor.plot_cards.delete(selected_card)

    @board.transaction do
      @actor.save!
      @board.save!
      @board.log!( current_user, @board.sauron, 'plot_cards.play', { plot_card: selected_card } )
    end

    redirect_to plot_cards_discard_screen_path(@actor)
  end

  def discard_screen
    @used_slots = @board.current_plots
    @used_slots_options = @used_slots.keys.map{ |e| [ e.gsub( 'plot-card-'.freeze, 'Card slot '.freeze ), e ] }.sort
  end

  def discard
    @selected_card_slot = params[:selected_card]
    validate_card_slot_syntax
    discarded_card = @board.current_plots[@selected_card_slot]

    @board.plot_deck << discarded_card
    @board.current_plots.delete(@selected_card_slot)

    @board.transaction do
      @board.save!
      @board.log!( current_user, @board.sauron, 'plot_cards.discard', { plot_card: discarded_card } )
    end

    redirect_to plot_cards_discard_screen_path(@actor)
  end

  def draw_screen
    @cards_already_drawn = @actor.drawn_plot_cards
  end

  def draw
    GameEngine::Deck.new(current_user, @board, @actor, DECK_NAME ).draw_cards(params[:nb_cards])
    redirect_to plot_cards_keep_screen_path(@actor)
  end

  def keep_screen
    @cards = @actor.drawn_plot_cards
  end

  def keep
    GameEngine::Deck.new(current_user, @board, @actor, DECK_NAME ).keep_cards(
        params[:selected_cards].split(',').map{ |e| e.to_i } )
    redirect_to plot_cards_play_screen_path(@actor)
  end

  private

  def validate_card_slot_syntax
    raise "Bad formatted card slot = #{@selected_card_slot}" unless @selected_card_slot =~ /plot-card-\d/
  end

end