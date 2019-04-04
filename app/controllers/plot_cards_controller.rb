class PlotCardsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor

  DECK_NAME = 'plot'.freeze

  def draw_screen
    @cards_already_drawn = @actor.drawn_plot_cards
    @nb_cards = params[:nb_cards] || 2
  end

  def draw
    ge = GameEngine::Deck.new(current_user, @board, @actor, DECK_NAME, discard_card_action: :back_to_bottom )
    ge.draw_cards(params[:nb_cards])

    redirect_to plot_cards_keep_screen_path(@actor)
  end

  def keep_screen
    @cards = @actor.drawn_plot_cards
  end

  def keep
    cards = params[:selected_cards].split(',').map{ |e| e.to_i }

    ge = GameEngine::Deck.new(current_user, @board, @actor, DECK_NAME, discard_card_action: :back_to_bottom )
    ge.keep_cards(cards)

    redirect_to GameEngine::RouteFromBoardState.get_route(@board,@actor), notice: I18n.t( 'notices.plot_cards_keep_success', count: cards.count )
  end

  #
  # Play area
  #
  def play_screen
    @cards = @actor.plot_cards
    @free_slots = 1.upto(3).to_a - @board.current_plots.map{ |e| e.plot_position }

    @free_slots_options = @free_slots.map{ |e| [ "Card slot #{e}".freeze, e ] }.sort
  end

  def play
    @selected_card_slot = params[:card_slot]
    selected_card = params[:selected_card].to_i

    raise "Actor does not have card #{selected_card} in #{@actor.plot_cards}" unless @actor.plot_cards.include?(selected_card)
    raise "Slot already in use : #{@selected_card_slot} = #{@board.get_plot_card(@selected_card_slot)}" if @board.get_plot_card(@selected_card_slot)

    @board.set_plot(@actor, @selected_card_slot, selected_card )
    @actor.plot_cards.delete(selected_card)

    @board.transaction do
      @actor.save!
      @board.save!
      @board.log( @board.sauron, 'plot_cards.play', { plot_card: selected_card } )
    end

    redirect_to plot_cards_play_screen_path(@actor), notice: 'Card successfuly played'
  end

  def play_finished
    @board.transaction do
      @board.next_to_sauron_actions!

      GameData::Events.new.draw_next_event_card(@board)

      redirect_to edit_sauron_action_path(@actor)
    end
  end

  #
  # Discard area
  #
  def discard_screen
    @used_slots = @board.current_plots
    @used_slots_options = @used_slots.map{ |e| [ "Card slot #{e.plot_position}".freeze, e.plot_position ] }.sort
    @selection = 'selectable-card-selection-unique'
  end

  def discard
    @selected_card_slot = params[:selected_card]
    discarded_card = @board.get_plot_card @selected_card_slot

    @board.plot_deck << discarded_card
    @board.current_plots.where( plot_position: @selected_card_slot ).delete_all

    @board.transaction do
      @board.save!
      @board.log( @board.sauron, 'plot_cards.discard', { plot_card: discarded_card } )
    end

    redirect_to plot_cards_discard_screen_path(@actor)
  end

end