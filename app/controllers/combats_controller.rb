class CombatsController < ApplicationController

  before_action :require_logged_in
  before_action :set_combat
  before_action :set_actor_ensure_actor, only: [:play_combat_card_screen, :play_combat_card_hero, :play_combat_card_mob]

  def show
    @hero_used_strength = 0
    @mob_used_strength = 0
  end

  def play_combat_card_screen
    @selectable_card_class = 'selectable-card-selection-unique'
  end

  def play_combat_card_hero
    @actor = @combat.hero
    @actor.combat_card_played = params[:selected_card].to_i
    @actor.save!
    @board.set_hero_activation_state(@actor, false)
    resolve_combat
  end

  def play_combat_card_mob
    @mob.combat_card_played = params[:selected_card].to_i
    @mob.save!
    @board.set_sauron_activation_state(false)
    resolve_combat
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

      @hero.combat_temporary_strength = temporary_strength
      @hero.save!
      
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
        @hero.combat_cards_played << @hero.combat_card_played
        @mob.combat_cards_played << @mob.combat_card_played

        @board.next_to_apply_damages_board_combats!

        @board.set_hero_activation_state( @hero, true )
        @board.set_sauron_activation_state( true )

        redirect_to apply_damages_board_combats_path(@board, @actor)
      else
        redirect_to boards_path
      end
    end
end
