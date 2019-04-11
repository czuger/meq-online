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
    @board.next_to_movement!

    redirect_to hero_movement_screen_path(@actor)
  end

  #
  # Movement methods
  #
  def movement_screen
    set_heroes_hero_and_locations
  end

  def move
    @actor.transaction do
      @actor.location = params[:button]

      selected_cards = params[:selected_cards].split(',').map(&:to_i)

      if selected_cards - @actor.hand != []
        raise "Selected cards not in hand. selected_cards = #{selected_cards}, hand = #{@actor.hand}"
      end

      @actor.hand_to_rest(selected_cards)
      @actor.save!

      @actor.suffer_peril!(@board)

      @board.next_to_exploration!
      @board.save!

      redirect_to hero_exploration_screen_path(@actor)
    end
  end

  def movement_finished
    @board.transaction do
      @board.next_to_exploration!
      @board.save!

      redirect_to hero_exploration_screen_path(@actor)
    end
  end

  #
  # Exploration methods
  #
  def exploration_screen
    # Need to process that.
    @tokens_at_location = @board.get_tokens_at_location(@actor.location)
  end

  def explore
    @board.transaction do

      params[:tokens].each do |type, elements|
        case type
          when 'character'
            elements.each do |character|
              @actor.favor += 2
              @board.characters.delete(character)
              @board.log( @actor, 'exploration.encounter_character', { location_name: @actor.location, character_name: character } )
            end
          when 'favor'
            elements.each do |_|
              @actor.favor += 1
              # Ensure that only one favor is removed at one time.
              @board.favors.slice!(@board.favors.index(@actor.location))
              @board.log( @actor, 'exploration.get_favor', { location_name: @actor.location } )
            end
        end
      end

      @actor.save!
      @board.save!
    end

    redirect_to hero_exploration_screen_path(@actor)
  end

  def exploration_back_to_movement
    @board.next_to_movement!
    @board.save!

    redirect_to hero_movement_screen_path(@actor)
  end

  def exploration_finished
    @board.transaction do
      # If we have more than one player
      if @board.current_heroes_count > 1
        @board.finish_heroes_turn!
        redirect_to boards_path
      else
      # If we have only one player
        if @board.start_hero_second_turn(@actor)
          # We started a new turn for hero
          redirect_to hero_draw_cards_screen_path(@actor)
        else
          redirect_to boards_path
        end
      end
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
    @board.transaction do
      @board.finish_hero_turn!
      redirect_to boards_path
    end
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

      # We switch to the first hero to play an switch to sauron shadow card play turn
      @board.transaction do
        @board.set_first_hero_to_play
        @board.next_to_rest_step!
      end
    end

    redirect_to hero_rest_screen_path(@actor)
  end

  private

  def set_heroes_hero_and_locations
    @heroes = GameData::Heroes.new
    @heroes_hero = @heroes.get( @actor.name_code )

    @last_location = @actor.location

    @locations = GameData::LocationsPaths.new.get_connected_locations_for_select(@last_location)

    @selectable_card_class = 'selectable-card-selection-multiple'
  end

end
