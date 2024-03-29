class HeroesController < ApplicationController

  before_action :require_logged_in
  before_action :set_sauron_ensure_sauron, except: [:index, :new]

  def argalad_power_screen
    raise "Abuse of Argalad power : #{current_user.inspect}" unless @actor.name_code == 'argalad'

    @monsters = @actor.argalad_surrounding_monsters
    @actor.used_powers['argalad'] = true
    @actor.save!
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
    game_data_location = GameData::Locations.new
    @can_heal = game_data_location.can_heal?(@actor.location)
  end

  def rest_rest
    @actor.transaction do
      @actor.rest!
      after_rest_or_heal
    end
  end

  def rest_heal
    @actor.transaction do
      @actor.heal!
      after_rest_or_heal
    end
  end

  def rest_skip
    @board.transaction do
      @board.next_to_hero_movement_screen!
      redirect_to hero_movement_screen_path(@actor)
    end
  end

  def discard_corruption_card_screen
    @corruptions = @actor.discardable_corruption_cards
  end

  def discard_corruption_card
    card = params[:selected_cards].to_i
    raise "Card #{card} already in deck #{@board.corruption_deck.inspect}" if @board.corruption_deck.include?( card )

    @board.transaction do
      @board.corruption_deck << params[:selected_cards]
      Corruption.where( board: @board, card_code: card ).delete_all

      @actor.corruption_card_discarded_this_turn = true

      @board.save!
      @actor.save!

      redirect_to hero_rest_screen_path(@actor), alert: 'Corruption card successfully discarded.'
    end
  end

  def after_rest_advance_story_marker_screen
    # TODO : add logs
    @lowest_screens = []

    lowest_markers = [ @board.story_marker_ring, @board.story_marker_conquest, @board.story_marker_corruption ]
    min_marker = lowest_markers.min

    @lowest_screens << 'Ring' if @board.story_marker_ring == min_marker
    @lowest_screens << 'Conquest' if @board.story_marker_conquest == min_marker
    @lowest_screens << 'Corruption' if @board.story_marker_corruption == min_marker
  end

  def after_rest_advance_story_marker
    board_column = 'story_marker_' + params[:marker].downcase

    @board.transaction do
      @board.update( board_column => @board.send(board_column) + 1 )
      @board.next_to_hero_movement_screen!
      redirect_to hero_movement_screen_path(@actor)
    end
  end

  #
  # Movement methods
  #
  def movement_screen
    check_heroes_powers
    set_heroes_hero_and_locations
  end

  def move
    @actor.transaction do
      selected_cards = params[:selected_cards].split(',').map(&:to_i)

      unless validate_movement( selected_cards )
        redirect_to hero_movement_screen_path( @actor ), alert: 'Selected cards does not match requirements.'
      else
        @actor.location = params[:button]

        if selected_cards - @actor.hand != []
          raise "Selected cards not in hand. selected_cards = #{selected_cards}, hand = #{@actor.hand}"
        end

        @actor.hand_to_rest(selected_cards)
        peril_notice = @actor.suffer_peril(@board)
        @actor.save!
        @board.save!

        location_encounters = @board.check_location_encounters(@actor)
        case location_encounters
          when :empty
            redirect_to hero_movement_screen_path(@actor), notice: peril_notice
          when :monster
            @board.next_to_combat_setup_screen_board_combats!

            redirect_to combat_setup_screen_board_combats_path(@board)
          when :explorations
            @board.next_to_hero_exploration!

            redirect_to hero_exploration_path(@actor)
          else
            raise "location_encounters unknown : #{location_encounters}"
        end
      end

      RefreshChannel.refresh
    end
  end

  def movement_finished
    @board.transaction do
      @board.next_to_hero_exploration!
      @board.save!

      redirect_to hero_exploration_path(@actor)
    end
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
        check_rest
      end
    end

    redirect_to boards_path
  end

  private

  def finish_hero_turn
    @board.transaction do
      # If we have more than one player
      @board.hero_end_turn_operations(@actor)

      if @board.current_heroes_count > 1
        @board.finish_heroes_turn!(@actor)
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

  def check_heroes_powers
    if @actor.name_code == 'argalad' && !@actor.used_powers['argalad']
      # Check monsters near Argalad
      if @actor.argalad_surrounding_monsters.count >= 1
        @argalad_can_use_power = true
      end
    end
  end

  def after_rest_or_heal
    if @board.advance_lowest_story_marker( random: true )
      @board.next_to_hero_movement_screen!
      redirect_to hero_movement_screen_path(@actor)
    else
      @board.next_to_after_rest_advance_story_marker!
      redirect_to hero_after_rest_advance_story_marker_screen_path(@actor)
    end
  end

  def set_heroes_hero_and_locations
    @heroes = GameData::Heroes.new
    @heroes_hero = @heroes.get( @actor.name_code )

    @last_location = @actor.location

    @locations = GameData::LocationsPaths.new.get_connected_locations_for_select(@last_location, @actor.items['horse'])

    @selectable_card_class = 'selectable-card-selection-multiple'
  end

  def check_rest
    location_encounters = @board.check_location_encounters(@actor)
    game_data_location = GameData::Locations.new

    if location_encounters == :monster
      @board.next_to_combat_setup_screen_board_combats!
    elsif game_data_location.can_not_rest?( @actor.location )
      @board.next_to_hero_rest_screen!
      @board.next_to_hero_movement_screen!
    else
      @board.next_to_hero_rest_screen!
    end
  end

  def validate_movement( selected_cards )
    heroes_data = GameData::Heroes.new
    skills_data = GameData::Skills.new

    path_type, path_difficulty = GameData::LocationsPaths.new.path_data(@actor.location, params[:button] )

    path_difficulty = [path_difficulty-1,1].max if @actor.items['horse']

    selected_cards.each do |card|
      if card >= 100
        # This is a skill
        card = skills_data.get(card)
      else
        # This is a hero card
        card = heroes_data.get(@actor.name_code).cards[card]
      end

      # If the card is the same type than the path, then it is ok. We return true.
      return true if card.movement_type == path_type.to_sym

      path_difficulty -= 1
      # If we have the good number of cards, then we have it. We return true.
      return true if path_difficulty == 0
    end

    # If no conditions have been satisfied, then the cards does not match the path requirements. We return false.
    false
  end

end
