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
      @game_data_monsters ||= GameData::Mob.new
      @game_data_monsters.get(monster_code).name
    else
      'Nothing'
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
