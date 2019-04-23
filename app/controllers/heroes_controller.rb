class HeroesController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor, except: [:index, :new]


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
    location_encounters = @board.check_location_encounters(@actor)

    if location_encounters == :monster
      @board.next_to_combat_setup_screen_board_combats!
      @board.save!

      redirect_to combat_setup_screen_board_combats_path(@board)
    end
  end

  def rest_rest
    @actor.transaction do
      @actor.rest
      after_rest_or_heal
    end
  end

  def rest_heal
    @actor.transaction do
      @actor.heal
      after_rest_or_heal
    end
  end

  def rest_skip
    @board.transaction do
      @board.next_to_hero_movement_screen!
      redirect_to hero_movement_screen_path(@actor)
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
        @actor.save!

        @actor.suffer_peril!(@board)

        location_encounters = @board.check_location_encounters(@actor)
        case location_encounters
          when :empty
            redirect_to hero_movement_screen_path(@actor)
          when :monster
            @board.next_to_combat_setup_screen_board_combats!
            @board.save!

            redirect_to combat_setup_screen_board_combats_path(@board)
          when :exploration
            @board.next_to_exploration!
            @board.save!

            redirect_to hero_exploration_screen_path(@actor)
          else
            raise "location_encounters unknown : #{location_encounters}"
        end
      end
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
    notice = nil

    @board.transaction do
      params[:tokens].each do |type, elements|
        case type
          when 'character'
            elements.each do |character|
              @actor.favor += 2
              @board.characters.delete(character)
              @board.log( @actor, 'exploration.encounter_character', { location_name: @actor.current_location_name, character_name: character } )
            end
            notice = 'Character successfully encountered.'
          when 'favor'
            elements.each do |_|
              @actor.favor += 1
              # Ensure that only one favor is removed at one time.
              @board.favors.slice!(@board.favors.index(@actor.location))
              @board.log( @actor, 'exploration.get_favor', { location_name: @actor.current_location_name } )
            end
            notice = 'Favor successfully taken.'
          when 'plot'
            # For now we assume that there is only one plot at the location. If there is more than one, we get the first
            plot = @board.current_plots.where( affected_location: @actor.location ).first

            if plot.favor_to_discard <= @actor.favor
              @actor.favor -= plot.favor_to_discard
              plot.destroy!
              @board.log( @actor, 'exploration.plot.remove', { location_name: @actor.current_location_name } )
              notice = 'Plot successfully removed.'
            else
              raise "This shouldn't happens"
            end
        end
      end

      @actor.save!
      @board.save!
    end

    redirect_to hero_exploration_screen_path(@actor), notice: notice
  end

  def exploration_back_to_movement
    @board.next_to_hero_movement_screen!
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

  ###

  # def take_damages
  #   damage_amount = params[:damage_amount].to_i
  #   damages_taken_from_life_pool = @actor.life_pool.shift(damage_amount)
  #   @actor.damage_pool += damages_taken_from_life_pool
  #   rest_damages = damage_amount - damages_taken_from_life_pool.count
  #   @actor.damage_pool += @actor.hand.shift(rest_damages)
  #   @actor.save!
  #   redirect_to @actor
  # end

  def single_hero_draw
    redirect_to hero_draw_cards_screen_path(@actor)
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
        @board.next_to_hero_rest_screen!
      end
    end

    redirect_to hero_rest_screen_path(@actor)
  end

  private

  def after_rest_or_heal
    if advance_lowest_story_marker
      @board.next_to_hero_movement_screen!
      redirect_to hero_movement_screen_path(@actor)
    else
      @board.next_to_after_rest_advance_story_marker!
      redirect_to hero_after_rest_advance_story_marker_screen_path(@actor)
    end
  end

  # Try to advance the lowest story marker. If was able to do, return true, false otherwise.
  def advance_lowest_story_marker
    @lowest_markers = [ @board.story_marker_ring, @board.story_marker_conquest, @board.story_marker_corruption ]
    @min_marker = @lowest_markers.min

    lowests_markers_count = @lowest_markers.select{ |e| e == @min_marker }.count

    # If we have more than one lowest markers, we will have to ask to player
    return false if lowests_markers_count > 1

    @board.story_marker_ring += 1 if @board.story_marker_ring == @min_marker
    @board.story_marker_conquest += 1 if @board.story_marker_conquest == @min_marker
    @board.story_marker_corruption += 1 if @board.story_marker_corruption == @min_marker
    @board.save!

    true
  end

  def set_heroes_hero_and_locations
    @heroes = GameData::Heroes.new
    @heroes_hero = @heroes.get( @actor.name_code )

    @last_location = @actor.location

    @locations = GameData::LocationsPaths.new.get_connected_locations_for_select(@last_location, @actor.items['horse'])

    @selectable_card_class = 'selectable-card-selection-multiple'
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
