class ShadowCardsDrawController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor, only: [:edit, :update]

  def edit
    @cards = @actor.drawn_shadow_cards
  end

  def update
    case params[:button]
    when 'draw'
      draw_shadow_cards
    when 'keep'
      keep_shadow_cards
    when 'discard'
      discard
    else
      raise "Unknown order : #{params[:button]}"
    end

    redirect_to edit_shadow_cards_draw_path(@actor)
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

  def discard
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
    selected_cards = JSON.parse(params[:selected_cards]).map{ |e| e.to_i }

    raise "Selected cards should not be empty" if selected_card.empty?
    raise "Selected cards should be included in pool cards" unless (selected_cards - @actor.drawn_shadow_cards).empty?

    @actor.drawn_shadow_cards -= selected_cards
    @actor.shadow_cards += selected_cards

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