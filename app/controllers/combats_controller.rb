class CombatsController < ApplicationController

  before_action :require_logged_in
  before_action :set_combat
  before_action :set_actor_ensure_actor, only: [:play_combat_card_screen]

  def show
    @hero_used_strength = @combat.hero_strength_used
    @mob_used_strength = @combat.mob_strength_used

    @last_hero_cards_used = @combat.combat_card_played_heroes.last(6)
    @last_mob_cards_used = @combat.combat_card_played_mobs.last(6)

    @print_hero_play_link = current_user?( @hero )
    @print_sauron_play_link = current_user?( @board.sauron )
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
    @actor = @hero
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

  def play_combat_card( me, secret_played_card )
    @combat.transaction do
      @actor = me.kind_of?(Mob) ? @board.sauron : me

      # Remove player card from hero hand
      hand = me.hand
      raise "Card #{secret_played_card} not in #{hand}" unless hand.include?(secret_played_card)
      hand.slice!(hand.index(secret_played_card))
      me.hand = hand
      me.save!
      @combat.save!

      resolve_played_combats_cards
    end
  end

  # Use callbacks to share common setup or constraints between actions.
  def set_combat
    @board = Board.find(params[:board_id])
    @combat = @board.combat
    @hero = @combat.hero
    @mob = @combat.mob
  end

  def resolve_played_combats_cards
    if @hero.active == false && @board.sauron.active == false

      result = @combat.reveal_secretly_played_cards

      # if result.mob_defeated || result.hero_defeated || (result.mob_exhausted && result.hero_exhausted)
      #   resolve_combat(result)
      # else
        @board.set_hero_activation_state( @hero, true )
        @board.set_sauron_activation_state( true )

        redirect_to board_combats_path(@board)
      # end
    else
      redirect_to boards_path
    end
  end

  # Monster defeated -> Remove monster, Continue movement
  # Hero defeated -> Remove monster, Place influence, Stop movement, Heal
  # Both defeated -> Remove monster, Stop movement, Heal
  # Monster exauhsted -> Continue combat
  # Hero exauhsted -> Continue combat
  # Both exauhsted -> Remove monster, Place influence, Stop movement
  # Else -> Continue combat
  def resolve_combat(result)
    discard_cards

    # This will be a step for Sauron
    place_influence_amount unless result.mob_defeated

    # This will be a step for Sauron
    defeate_hero if result.hero_defeated

    @combat.destroy!
    @mob.destroy! if @mob.is_a?( Monster )
  end

  def discard_cards
    @hero.rest_pool += @combat.combat_card_played_heroes.map{ |c| c.card }
    @hero.save!
  end
end
