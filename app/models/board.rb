require 'ostruct'

class Board < ApplicationRecord

  include GameEngine::BoardAasm
  include GameEngine::ActorActivation
  include GameEngine::ActorTurnManagement

  has_many :logs, dependent: :destroy

  has_many :heroes, dependent: :destroy
  has_one :sauron, dependent: :destroy

  has_many :current_plots, class_name: 'BoardPlot', dependent: :destroy

  has_and_belongs_to_many :users

  belongs_to :current_hero, class_name: 'Hero', optional: true

  has_many :mobs, dependent: :destroy
  has_many :monsters
  has_many :minions

  has_one :combat

  serialize :characters
  serialize :favors
  serialize :sauron_actions
  serialize :influence

  serialize :plot_deck
  serialize :plot_discard

  serialize :shadow_deck
  serialize :shadow_discard

  serialize :corruption_deck
  serialize :corruption_discard

  serialize :event_deck
  serialize :event_discard

  serialize :monsters_pool_orange
  serialize :monsters_pool_purple
  serialize :monsters_pool_dark_blue
  serialize :monsters_pool_brown
  serialize :monsters_pool_dark_green

  #
  # Create a new board
  #
  def self.create_new_board(max_heroes_count=4)

    starting_plot_id= rand( 0..2 )
    starting_plot = GameData::Plots.new.get(starting_plot_id)

    plot_deck= ((3..17).to_a - [10, 11, 14, 15, 17]).to_a.shuffle

    # We currently remove shadow cards that are played during hero movement.
    shadow_deck= ((0..23).to_a - [1, 21]).shuffle
    event_deck = GameData::Events.new.get_starting_event_deck

    max_heroes_count= max_heroes_count

    location_monsters = GameData::LocationsMonsters.new

    board = Board.new(
      influence: starting_plot.influence.init,
      plot_deck: plot_deck,
      shadow_deck: shadow_deck,
      event_deck: event_deck,
      plot_discard: [],
      shadow_discard: [],
      corruption_discard: [],
      event_discard: [],
      favors: [],
      max_heroes_count: max_heroes_count,
      shadow_pool: starting_plot.influence.shadow_pool,
      characters: {},
      corruption_deck: GameData::CorruptionCards.new.deck.shuffle,
      sauron_actions: [],
      monsters_pool_orange: location_monsters.get_deck(:orange),
      monsters_pool_purple: location_monsters.get_deck(:purple),
      monsters_pool_dark_blue: location_monsters.get_deck(:dark_blue),
      monsters_pool_brown: location_monsters.get_deck(:brown),
      monsters_pool_dark_green: location_monsters.get_deck(:dark_green)
    )

    return board, starting_plot, starting_plot_id
  end
  #
  # Create combat method
  #
  def check_location_encounters(actor)
    tokens_at_location = get_tokens_at_location(actor.location)
    tokens_at_location.delete_if{ |e| e.type == :hero }

    # We go to the exploration screen only if we found something on the location (other than a hero of course)
    if tokens_at_location.empty?
      :empty
    else
      # We fight the hardest monster first
      monster_at_location = mobs.where( location: actor.location ).order( 'life DESC, strength DESC' ).first

      if monster_at_location
        if monster_at_location.code == 'nothing'
          log( actor, 'combat.encounter_nothing', location: actor.location, hero: actor.name )

          monster_at_location.destroy!

          :empty
        else
          create_combat( actor, monster_at_location )

          :monster
        end
      else
        :explorations
      end
    end
  end

  def create_combat( hero, mob )
    mob.transaction do
      cards = GameData::MobsCards.new.get_deck(mob.attack_deck)
      mob.hand = cards.shift( mob.fortitude )
      mob.save!
      create_combat!( hero: hero, mob: mob )
    end
  end

  #
  # Location methods
  #
  def location_name( location_code )
    @game_data_locations ||= GameData::Locations.new
    @game_data_locations.get(location_code).name
  end

  def location_exists?( location_code )
    @game_data_locations ||= GameData::Locations.new
    @game_data_locations.exist?(location_code)
  end

  #
  # Monster methods
  #
  def create_monster( monster_code, location, pool_key= nil )

    raise 'monster_code is a mandatory parameter' unless monster_code

    starting_deck = []
    if monster_code.to_s == 'nothing'
      monster_data = OpenStruct.new( fortitude: -1, strength: -1, life: -1, name: 'Nothing', attack_deck: :none )
    else
      @game_data_monsters ||= GameData::Mobs.new
      monster_data = @game_data_monsters.get(monster_code)
    end

    monster_hash = { location: location, code: monster_code, fortitude: monster_data.fortitude,
                     strength: monster_data.strength, life: monster_data.life, name: monster_data.name,
                     attack_deck: monster_data.attack_deck, max_life: monster_data.life, hand: [] }

    if monster_data.type == :minion
      minions.create!( monster_hash )
    else
      monster_hash[:pool_key] = pool_key
      monsters.create!( monster_hash )
    end

  end

  #
  # Tokens methods
  #
  def get_tokens
    GameEngine::DataAtLocation.new.gather(self).tokens
  end

  def get_tokens_at_location(location)
    GameEngine::DataAtLocation.new.gather(self).tokens[location.to_s]
  end

  #
  # Log methods
  #
  def log( actor, action, params= {} )
    # logs.create!( board: self, actor: actor, action: action, params: params )
  end

  #
  # Plots methods
  #
  def set_plot( actor, plot_position, plot_id, starting_plot = nil )
    actor.assert_sauron if actor

    plot = starting_plot || GameData::Plots.new.get(plot_id)

    # Protection agains multiple plots is done at functional level.
    self.current_plots.create!( plot_position: plot_position, plot_card: plot_id, affected_location: plot.affect,
                                story_type: plot.story.type, story_advance: plot.story.advance,
                                favor_to_discard: plot.favor_to_discard )

    log( actor, :place_plot )
  end

  def get_plot_card( plot_position )
    pp = current_plots.where( plot_position: plot_position ).first
    pp && pp.plot_card
  end

  def discard_plot( actor, plot_position )
    actor.assert_sauron if actor
    # logs.create!( board: self, actor: actor, action: action, params: params )
  end

end
