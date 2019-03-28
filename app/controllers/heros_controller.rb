class HerosController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor, except: [:index, :new]

  def index
    @board = Board.find(params[:board_id])
    @heros = @board.heroes.select{ |h| h.user_id == current_user.id }
  end

  def show
    if @actor.active
      @heroes = GameData::Heroes.new
      @heroes_hero = @heroes.get( @actor.name_code )

      @locations = GameData::Locations.new
      @locations.delete!(@actor.location)

      @cards = @actor.hand
    else
      redirect_to board_inactive_actor_path(@board)
    end
  end

  #
  # Rest methods
  #
  def rest_screen
  end

  def rest
    if params[:rest]
      @actor.rest
    elsif params[:heal]
      @actor.heal
    end

    redirect_to hero_rest_screen_path(@actor)
  end

  def rest_finished
    @board.next_to_movement_preparation_step!

    redirect_to hero_movement_preparation_steps_path(@actor)
  end

  #
  # Movement methods
  #
  def movement_screen
  end

  def move
  end

  def movement_finished
    @board.transaction do
      @board.next_to_exploration!
      @board.switch_to_current_hero
      @board.save!

      redirect_to hero_exploration_screen_path(@actor)
    end
  end

  #
  # Exploration methods
  #
  def exploration_screen
  end

  def explore
    @board.next_to_movement!
    @board.save!

    redirect_to hero_movement_screen_path(@actor)
  end

  def exploration_finished
    @board.transaction do
      @board.next_to_movement!
      @board.switch_to_current_hero
      @board.save!

      redirect_to boards_path
    end
  end

  #
  # Encounter methods
  #
  def encounter_screen
  end

  def encounter
  end

  def encounter_finished
    # At this step we need to switch to next player, and if all players have finished theire turn
    # We need to skip to Sauron turn.
    # @board.transaction do
    #   @board.next_to_exploration!
    #   @board.switch_to_current_hero
    #   @board.save!
    #
    #   redirect_to hero_exploration_screen_path(@actor)
    # end
  end

  ###

  def take_damages
    damage_amount = params[:damage_amount].to_i
    damages_taken_from_life_pool = @actor.life_pool.shift(damage_amount)
    @actor.damage_pool += damages_taken_from_life_pool
    rest_damages = damage_amount - damages_taken_from_life_pool.count
    @actor.damage_pool += @actor.hand.shift(rest_damages)
    @actor.save!
    redirect_to @actor
  end

  def finish_turn
    @actor.transaction do
      @actor.update( turn_finished: true )
      @board.log( @actor, :finish_turn )
    end
    redirect_to @actor
  end

  # def move
  #   hero_location = @actor.location
  #   @actor.location = params['move_to']
  #   card = params['card_used'].to_i
  #
  #   card_position = @actor.hand.index( card )
  #   if card_position
  #     @actor.hand.delete_at( card_position )
  #
  #     @actor.rest_pool << card
  #     @actor.save!
  #
  #     @actor.log_movement!( @board, card )
  #   else
  #     raise "Can't find a card position. card = #{card.inspect}, hand = #{@actor.hand.inspect}"
  #   end
  #
  #   redirect_to @actor
  # end

  def draw_cards_screen
    # nb_cards_to_draw = params[:nb_cards].to_i
    @cards = @actor.hand

    @heroes = GameData::Heroes.new
    @heroes_hero = @heroes.get( @actor.name_code )
  end

  def draw_cards
    nb_cards_to_draw = params[:nb_cards].to_i
    cards = @actor.life_pool.shift(nb_cards_to_draw)
    @actor.hand += cards
    @actor.save!

    # @actor.log_draw_cards!( @board, cards.count)

    redirect_to hero_draw_cards_screen_path(@actor)
  end

  def draw_cards_finished
    @board.set_hero_activation_state(@actor, false)

    # If no more heroes are actives, then we goes to next step
    unless @board.heroes_actives?

      @board.heroes.each do |hero|

        raise "Board #{@board.id} : hero.playing order should not be nil" unless hero.playing_order

        # If all heroes have finished their draw step, we find the first hero to play and we activate him
        if hero.playing_order == 0
          @board.transaction do
            @board.current_hero = hero
            @board.set_all_actors_activation_state(false)
            @board.set_sauron_activation_state( true)
            @board.save!

            #TODO : next step here

            @board.next_to_play_shadow_card_at_start_of_hero_turn!

            break
          end
        end
      end
    end

    redirect_to :boards

  end

end
