class SauronController < ApplicationController
  before_action :require_logged_in
  before_action :set_actor_ensure_actor

  def show
    GameEngine::RedirectFromBoardState.redirect(@board, @actor ){ |r| redirect_to r }
  end

  def setup
    @plot_card = GameData::StartingPlots.new.get(@board.current_plots['plot-card-1'])
  end

  # def shadow_cards
  #   if params[:button]=='draw'
  #     nb_cards = params[:nb_shadow_cards].to_i
  #     cards = @board.shadow_deck.shift(nb_cards)
  #     @actor.shadow_cards+= cards
  #
  #     @board.transaction do
  #       @actor.save!
  #       @board.save!
  #       @board.log!( current_user, @board.sauron, :draw_shadow_cards, { count: cards.count } )
  #     end
  #   end
  #   redirect_to @actor
  # end

end
