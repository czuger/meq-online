class ShadowCardsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor, only: [:edit, :update]

  def edit
    @drawn_cards = @actor.drawn_shadow_cards
    @cards_in_hand = @actor.shadow_cards
  end

  def update
    case params[:button]
    when 'draw'
      draw_shadow_cards
    when 'keep'
      keep_shadow_cards
    when 'discard_drawn_cards'
      discard_drawn_cards
    when 'discard_card_in_hand'
      discard_card_in_hand
    else
      raise "Unknown order : #{params[:button]}"
    end

    redirect_to edit_shadow_card_path(@actor)
  end

  private

  # def back_to_bottom_of_deck
  #   cards_count = @actor.drawn_shadow_cards.count
  #   @board.plot_deck += @actor.drawn_shadow_cards
  #   @actor.drawn_shadow_cards= []
  #
  #   @board.transaction do
  #     @board.save!
  #     @actor.save!
  #     @board.log!( current_user, @board.sauron, :cards_bottom_plot_deck, { count: cards_count } )
  #   end
  # end
  #

  def discard_card_in_hand
    cards_count = 1
    selected_card = params[:selected_card].to_i
    pp @actor.shadow_cards, selected_card
    @actor.shadow_cards.delete(selected_card)

    @board.transaction do
      @board.save!
      @actor.save!
      @board.log!( current_user, @board.sauron, 'shadow_cards.discard', { count: cards_count } )
    end
  end

  def discard_drawn_cards
    cards_count = @actor.drawn_shadow_cards.count
    @board.shadow_discard += @actor.drawn_shadow_cards
    @actor.drawn_shadow_cards= []

    @board.transaction do
      @board.save!
      @actor.save!
      @board.log!( current_user, @board.sauron, 'shadow_cards.discard', { count: cards_count } )
    end
  end

  def keep_shadow_cards
    selected_cards = params[:selected_cards].split(',').map{ |e| e.to_i }

    raise "Selected cards should not be empty" if selected_cards.empty?
    raise "Selected cards should be included in pool cards" unless (selected_cards - @actor.drawn_shadow_cards).empty?

    @actor.shadow_cards += selected_cards

    @board.shadow_discard += ( @actor.drawn_shadow_cards - selected_cards )
    @actor.drawn_shadow_cards = []

    @board.transaction do
      @board.save!
      @actor.save!
      @board.log!( current_user, @board.sauron, 'shadow_cards.keep', { count: selected_cards.count } )
    end
  end

  def draw_shadow_cards
    nb_cards = params[:nb_cards].to_i
    @cards = @board.shadow_deck.shift(nb_cards)
    @actor.drawn_shadow_cards= @cards

    @board.transaction do
      @actor.save!
      @board.save!
      @board.log!( current_user, @board.sauron, 'shadow_cards.draw', { count: @cards.count } )
    end
  end

end