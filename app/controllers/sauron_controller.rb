class SauronController < ApplicationController
  before_action :require_logged_in
  before_action :set_sauron, only: [:show,:shadow_cards]

  def show
    @player = @sauron
    @nb_cards = @sauron.drawn_plot_cards && !@sauron.drawn_plot_cards.empty? ? nil : 2
  end

  def shadow_cards
    if params[:button]=='draw'
      nb_cards = params[:nb_shadow_cards].to_i
      cards = @board.shadow_deck.shift(nb_cards)
      @sauron.shadow_cards+= cards

      @board.transaction do
        @sauron.save!
        @board.save!
        @board.log!( current_user, @board.sauron, :draw_shadow_cards, { count: cards.count } )
      end
    end
    redirect_to @sauron
  end

  private

    # Use callbacks to share common setup or constraints between actions.
    def set_sauron
      @sauron = Sauron.find(params[:sauron_id]||params[:id])
      ensure_sauron
      @board = @sauron.board
    end

end
