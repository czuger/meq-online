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
    starting_deck = []
    if monster_code.to_s == 'nothing'
      monster_data = OpenStruct.new( fortitude: -1, strength: -1, life: -1, name: 'Nothing', attack_deck: :none )
    else
      @game_data_monsters ||= GameData::Mobs.new
      monster_data = @game_data_monsters.get(monster_code)
    end

    monster_hash = { location: location, code: monster_code, fortitude: monster_data.fortitude,
                     strength: monster_data.strength, life: monster_data.life, name: monster_data.name,
                     attack_deck: monster_data.attack_deck, max_life: monster_data.life }

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
