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
      @combat.hero_secret_played_card = params[:selected_card].to_i
      @combat.save!

      # Remove player card from hero hand
      hand = @hero.hand
      hand.slice!(@hero.hand.index(@combat.hero_secret_played_card))
      @hero.hand = hand
      @hero.save!

      @board.set_hero_activation_state(@hero, false)
      resolve_combat
    end
  end

  def play_combat_card_mob
    @combat.transaction do
      @combat.mob_secret_played_card = params[:selected_card].to_i
      @combat.save!

      # Remove player card from hero hand
      hand = @mob.hand
      hand.slice!(@mob.hand.index(@combat.mob_secret_played_card))
      @mob.hand = hand
      @mob.save!

      @board.set_sauron_activation_state(false)
      resolve_combat
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

        redirect_to apply_damages_board_combats_path(@board, @actor)
      else
        redirect_to boards_path
      end
    end
end
