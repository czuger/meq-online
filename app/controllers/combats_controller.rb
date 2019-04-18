class CombatsController < ApplicationController

  before_action :require_logged_in
  before_action :set_combat
  before_action :set_actor_ensure_actor, only: [:play_combat_card_screen]

  def show
    @hero_used_strength = 0
    @mob_used_strength = 0
  end

  def play_combat_card_screen
    @selectable_card_class = 'selectable-card-selection-unique'
  end

  def play_combat_card_hero
    @combat.transaction do
      secret_played_card = params[:selected_card].to_i
      @combat.hero_secret_played_card = secret_played_card
      @board.set_hero_activation_state(@hero, false)

      play_combat_card( @hero, secret_played_card )
    end
  end

  def play_combat_card_mob
    @combat.transaction do
      secret_played_card = params[:selected_card].to_i
      @combat.mob_secret_played_card = secret_played_card
      @board.set_sauron_activation_state(false)

      play_combat_card( @mob, secret_played_card )
    end
  end

  def apply_damages
  end

  def combat_setup_screen
  end

  def combat_setup
    temporary_strength = @hero.strength

    @combat.transaction do
      if params[:button] == 'draw'
        @hero.draw_cards( @board, @hero.agility, true )
      else
        temporary_strength += @hero.agility

        @board.log( @hero, 'combat.inc_strength',name: @hero.name_code, str: @hero.strength,
                    nstr: temporary_strength )
      end

      @combat.temporary_hero_strength = temporary_strength
      @combat.save!
      
      @board.next_to_play_combat_card_screen_board_combats!

      @board.set_hero_activation_state( @hero, true )
      @board.set_sauron_activation_state( true )
      redirect_to play_combat_card_screen_board_combats_path( @board, @hero )
    end
  end

  private

  def play_combat_card( opponent, secret_played_card )
    @combat.transaction do
      # Remove player card from hero hand
      hand = opponent.hand
      hand.slice!(hand.index(secret_played_card))
      opponent.hand = hand
      opponent.save!
      @combat.save!

      resolve_combat
    end
  end

    # Use callbacks to share common setup or constraints between actions.
    def set_combat
      @board = Board.find(params[:board_id])
      @combat = @board.combat
      @hero = @combat.hero
      @actor = @hero
      @mob = @combat.mob
    end

    def resolve_combat
      if @hero.active == false && @board.sauron.active == false

        @combat.reveal_secretly_played_cards

        @board.set_hero_activation_state( @hero, true )
        @board.set_sauron_activation_state( true )

        redirect_to play_combat_card_screen_board_combats_path(@board, @actor)
      else
        redirect_to boards_path
      end
    end
end
