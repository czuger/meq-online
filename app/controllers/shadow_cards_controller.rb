class ShadowCardsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor

  DECK_NAME = 'shadow'.freeze

  def draw_screen
    @cards_already_drawn = @actor.drawn_shadow_cards
    @nb_cards = params[:nb_cards] || 2
  end

  def draw
    ge = GameEngine::Deck.new(current_user, @board, @actor, DECK_NAME, discard_card_action: :discard )
    ge.draw_cards(params[:nb_cards])

    redirect_to keep_screen_sauron_shadow_cards_url(@actor)
  end

  def keep_screen
    @cards = @actor.drawn_shadow_cards
  end

  def keep
    cards = params[:selected_cards].split(',').map{ |e| e.to_i }

    ge = GameEngine::Deck.new(current_user, @board, @actor, DECK_NAME, discard_card_action: :discard )
    ge.keep_cards(cards)

    redirect_to GameEngine::RouteFromBoardState.get_route(@board,@actor), notice: I18n.t( 'notices.shadow_cards_keep_success', count: cards.count )
  end

  def play_screen
    @cards = @actor.shadow_cards
  end

  def play
    selected_card = params[:selected_card].to_i

    raise "Actor does not have card #{selected_card} in #{@actor.shadow_cards}" unless @actor.shadow_cards.include?(selected_card)

    @board.shadow_discard << selected_card
    @actor.shadow_cards.delete(selected_card)

    @board.transaction do

      # We can play shadow card from many steps
      if @board.play_shadow_card_at_start_of_hero_turn?
        @board.next_to_rest_step!
      end

      @actor.save!
      @board.save!
      @board.log( @board.sauron, 'shadow_cards.play', { shadow_card: selected_card } )
    end

    redirect_to GameEngine::RouteFromBoardState.get_route(@board,@actor)
  end

  def start_hero_turn_play_card_screen
    @cards = @actor.shadow_cards
  end

  def start_hero_turn_play_card
    selected_card = params[:selected_card].to_i

    raise "Actor does not have card #{selected_card} in #{@actor.shadow_cards}" unless @actor.shadow_cards.include?(selected_card)

    @board.shadow_discard << selected_card
    @actor.shadow_cards.delete(selected_card)

    @board.transaction do
      @actor.save!
      @board.save!
      @board.log( @board.sauron, 'shadow_cards.play', { shadow_card: selected_card } )
    end

    redirect_to start_hero_turn_play_screen_sauron_shadow_cards_path(@actor)
  end

  def start_hero_turn_play_card_finished

    @board.transaction do
      @board.switch_to_current_hero

      @board.next_to_rest_step!

      redirect_to boards_path
    end

  end

end