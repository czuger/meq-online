class PlotCardsController < ApplicationController

  before_action :require_logged_in
  before_action :set_sauron_ensure_sauron

  DECK_NAME = 'plot'.freeze

  def draw_screen
    @cards_already_drawn = @actor.drawn_plot_cards
    @nb_cards = params[:nb_cards] || 2
  end

  def draw
    if @actor.drawn_plot_cards.empty?
      ge = GameEngine::Deck.new(current_user, @board, @actor, DECK_NAME, discard_card_action: :back_to_bottom )
      ge.draw_cards(params[:nb_cards])
    end

    redirect_to keep_screen_sauron_plot_cards_path(@actor)
  end

  def keep_screen
    @cards = @actor.drawn_plot_cards
  end

  def keep
    cards = params[:selected_cards].split(',').map{ |e| e.to_i }

    ge = GameEngine::Deck.new(current_user, @board, @actor, DECK_NAME, discard_card_action: :back_to_bottom )
    ge.keep_cards(cards)

    redirect_to GameEngine::RouteFromBoardState.new.get_route(@board,@actor), notice: I18n.t( 'notices.plot_cards_keep_success', count: cards.count )
  end

  #
  # Play area
  #
  def play_screen
    game_data = GameData::Plots.new

    @playable_cards = @actor.plot_cards.to_a.select{ |e| game_data.requirement( @board,e ) }
    @unplayable_cards = @actor.plot_cards - @playable_cards

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
      @board.log( @board.sauron, 'plot_cards.play', { plot_card: selected_card } )

      remove_gollum_cards( selected_card )

      @actor.save!
      @board.save!

      unless selected_card == 13
        play_finished
      else
        @board.next_to_look_for_gollum_cards_sauron_plot_cards
        redirect_to look_for_gollum_cards_sauron_plot_cards_path(@actor)
      end
    end
  end

  def play_finished
    @board.transaction do
      @board.next_to_edit_sauron_sauron_actions!

      GameData::Events.new.draw_next_event_card(@board)

      redirect_to edit_sauron_sauron_actions_path(@actor)
    end
  end

  def look_for_gollum_cards
    gollum_cards = [ 5, 12 ].freeze
    @gollum_cards_found = []

    @gollum_cards_found += gollum_cards & @board.plot_deck
    @gollum_cards_found += gollum_cards & @board.plot_discard
  end

  def get_gollum_card
    selected_card = params[:selected_card].to_i

    @actor.plot_cards << selected_card

    deck = @board.plot_deck + @board.plot_discard
    deck.delete( selected_card )
    deck.shuffle!

    @board.plot_discard = []
    @board.plot_deck = deck

    @board.transaction do
      @board.save!
      @actor.save!

      play_finished
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

    redirect_to play_screen_sauron_plot_cards_path(@actor)
  end

  def remove_gollum_cards( selected_card )
    gollum_cards = [ 5, 12 ]

    if gollum_cards.include?( selected_card )
      gollum_cards.delete( selected_card )
      to_discard_card = gollum_cards.first
      to_discard_card = @board.current_plots.where( plot_card: to_discard_card ).first

      if to_discard_card
        @board.log( nil, 'plot_cards.discard_gollum', { plot_card: to_discard_card } )
        @board.plot_discard << to_discard_card.plot_card
        to_discard_card.destroy!
      end
    end
  end

end