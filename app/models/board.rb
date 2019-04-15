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

  #
  # Create combat method
  #
  def create_combat( hero, mob )
    create_combat!( hero: hero, mob: mob )
  end

  #
  # Location methods
  #
  def location_name( location_code )
    @game_data_locations ||= GameData::Locations.new
    @game_data_locations.get(location_code).name
  end

  #
  # Monster methods
  #
  def monster_name( monster_code )
    if monster_code != 'nothing'
      @game_data_monsters ||= GameData::Mobs.new
      @game_data_monsters.get(monster_code).name
    else
      'Nothing'
    end
  end

  def create_monster( monster_code, location, pool_key= nil )
    if monster_code.to_s == 'nothing'
      monster_data = OpenStruct.new( fortitude: -1, strength: -1, life: -1, name: 'Nothing', starting_deck: [] )
    else
      game_data_monsters ||= GameData::Mobs.new
      monster_data = game_data_monsters.get(monster_code)
    end

    monster_hash = { location: location, code: monster_code, fortitude: monster_data.fortitude,
                     strength: monster_data.strength, life: monster_data.life, name: monster_data.name,
    hand: monster_data.starting_deck }

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
    logs.create!( board: self, actor: actor, action: action, params: params )
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
