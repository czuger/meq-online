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
      @actor.location = params[:destination]

      selected_cards = params[:selected_cards].split(',').map(&:to_i)

      if selected_cards - @actor.hand != []
        raise "Selected cards not in hand. selected_cards = #{selected_cards}, hand = #{@actor.hand}"
      end
      @actor.hand -= selected_cards
      @actor.rest_pool += selected_cards
      @actor.save!

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
  end

  def explore
    @board.next_to_movement!
    @board.save!

    redirect_to hero_movement_screen_path(@actor)
  end

  def exploration_finished
    # - 1 hero, hero.turn == 1 => hero second turn
    # - 1 hero, hero.turn == 2 => sauron turn
    # - 2-3 heroes => hero second turn
    # - 1 hero, hero.turn == 2 => sauron turn
    @board.transaction do
      if @board.current_heroes_count > 1
        @board.finish_heroes_turn!
        redirect_to boards_path
      else

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

  #
  # Movement methods (with movement preparation step)
  #
  # def movement_screen
  #   @locations_data = GameData::Locations.new
  #   @next_movement = @actor.movement_preparation_steps.first
  #   if @next_movement
  #     @from = @locations_data.get( @next_movement.origine ).name
  #     @to = @locations_data.get( @next_movement.destination ).name
  #   end
  # end
  #
  # def move
  #   @next_movement = @actor.movement_preparation_steps.find(params[:movement_id].to_i)
  #
  #   @actor.transaction do
  #     @actor.location = @next_movement.destination
  #
  #     if @next_movement.selected_cards - @actor.hand != []
  #       raise "Selected cards not in hand. selected_cards = #{@next_movement.selected_cards}, hand = #{@actor.hand}"
  #     end
  #     @actor.hand -= @next_movement.selected_cards
  #     @actor.rest_pool += @next_movement.selected_cards
  #     @actor.save!
  #
  #     MovementPreparationStep.delete(@next_movement.id)
  #
  #     @board.next_to_exploration!
  #     @board.save!
  #
  #     redirect_to hero_exploration_screen_path(@actor)
  #   end
  # end
  #
  # def movement_finished
  #   @board.transaction do
  #     @board.next_to_exploration!
  #     @board.save!
  #
  #     redirect_to hero_exploration_screen_path(@actor)
  #   end
  # end

end
