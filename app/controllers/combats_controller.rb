class CombatsController < ApplicationController

  before_action :require_logged_in
  before_action :set_combat
  before_action :set_actor_ensure_actor, only: [:play_combat_card_screen]
  before_action :set_combat_result, only: [:show, :play_combat_card_screen, :terminate]

  def show
    @last_hero_cards_used = @combat.combat_card_played_heroes.last(6)
    @last_mob_cards_used = @combat.combat_card_played_mobs.last(6)

    @print_hero_play_link = current_user?( @hero ) && @hero.active && !@combat.hero_exhausted
    @print_sauron_play_link = current_user?( @board.sauron ) && @board.sauron.active && !@combat.mob_exhausted
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

  def cards_loss_screen
    @loss_mandatory = @hero.temporary_damages > 0 && @hero.life_pool.count <= 0
    @selectable_card_class = 'selectable-card-selection-multiple'
    @actor = @hero
  end

  def cards_loss
    selected_cards = params[:selected_cards].split(',').map(&:to_i)

    required_cards = selected_cards.shift( @hero.temporary_damages )

    @hero.temporary_damages -= required_cards.count
    @hero.hand_to_damages(required_cards)
    @hero.save!

    if @hero.temporary_damages > 0
      redirect_to cards_loss_screen_board_combats_path(@board)
    else
      @board.next_to_play_combat_card_screen_board_combats!
      redirect_to board_combats_path(@board)
    end
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

  def terminate
    hero_turn_end = false

    @board.transaction do
      discard_cards
      destroy_combat

      if @combat_result.hero_defeated

        @board.advance_lowest_story_marker( random: true )
        @hero.favor = [ @hero.favor - 1, 0 ].max
        @hero.move_to_regional_haven
        @hero.heal

        @board.next_to_cards_loss_screen_board_combats!
        @board.next_to_after_defeat_advance_story_marker!
        hero_turn_end = true
      elsif @combat_result.mob_defeated
        # The hero continue his movement.
        @board.next_to_hero_movement_screen!
        @board.activate_current_hero

      elsif @combat_result.mob_exhausted && @combat_result.hero_exhausted
        @board.next_to_cards_loss_screen_board_combats!
        @board.next_to_after_defeat_advance_story_marker!
        hero_turn_end = true
      else
        raise "Shouldn't happen : #{@combat_result.inspect}"
      end

      if hero_turn_end
        if @board.finish_heroes_turn!(@hero) == :hero_draw_cards_screen
          redirect_to hero_draw_cards_screen_path(@hero)
        else
          redirect_to boards_path
        end
      else
        redirect_to hero_movement_screen_path( @hero )
      end
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

  def set_combat_result
    @hero_life = @hero.life_pool.count + @hero.hand.count
    @combat_result = OpenStruct.new( mob_defeated: @mob.life <= 0, hero_defeated: @hero_life <= 0,
                                     mob_exhausted: @combat.mob_exhausted, hero_exhausted: @combat.hero_exhausted )

    # There is a special exhaustion rule that does not apply when opponents play cards
    if @hero.hand.count <= 0
      @combat.hero_exhausted = true
      @combat_result.hero_exhausted = true
    end

    if @mob.hand.count <= 0
      @combat.mob_exhausted = true
      @combat_result.mob_exhausted = true
    end

    @hero.save!
    @mob.save!
    @combat.save!

    @hero_used_strength = @combat.hero_strength_used
    @mob_used_strength = @combat.mob_strength_used
  end

  def resolve_played_combats_cards
    if @hero.active == false && @board.sauron.active == false

      @combat.reveal_secretly_played_cards

      # Immediately apply damages
      required_cards = @hero.life_pool.shift( @hero.temporary_damages )
      @hero.temporary_damages -= required_cards.count
      @hero.damage_pool += required_cards
      @hero.save!

      if @hero.temporary_damages > 0
        if @hero.temporary_damages < @hero.life_pool.count + @hero.hand.count
          @board.next_to_cards_loss_screen_board_combats!
          redirect_to cards_loss_screen_board_combats_path(@board)
        else
          # The hero has been defeated.
          @hero.damage_pool += @hero.life_pool + @hero.hand
          @hero.life_pool.clear
          @hero.hand.clear
          @hero.save!
          redirect_to board_combats_path(@board)
        end
      else
        redirect_to board_combats_path(@board)
      end
    else
      redirect_to boards_path
    end
  end

  def destroy_combat

    unless Rails.env.test?
      File.open("log/combat_#{@combat.id}.json", 'w') do |f|
        c = {}
        c[:combat] = @combat
        c[:hero] = @combat.hero
        c[:mob] = @combat.mob
        c[:played_cards] = @combat.combat_card_playeds

        PP.pp(c.to_json,f)
      end
    end

    @combat.destroy!
    @mob.destroy! if @mob.is_a?( Monster )
  end

  def discard_cards
    @hero.rest_pool += @combat.combat_card_played_heroes.map{ |c| c.card }
    @hero.save!
  end
end
