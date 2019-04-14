class SauronMonstersController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor
  before_action :set_monster, only: [:edit, :update]

  def index
    @monsters = @board.monsters.order( 'location, code' )
  end

  def new
    @available_locations = @board.influence.map{ |k, v| k if v >= 1 }.compact.sort
    heroes_locations = @board.heroes.map{ |l| l.location }
    @available_locations -= heroes_locations
  end

  def create
    @game_data_locations_monsters = GameData::LocationsMonsters.new

    location = params[:location]

    # If location is crap, this should fail, so it is protected (against injections).
    # TODO : we need to handle an empty monster pool
    @game_data_locations_monsters.place_new_monster(@board, location )

    redirect_to sauron_sauron_monsters_path(@actor)
  end

  def edit
    @available_locations = GameData::LocationsPaths.new.get_connected_locations(@monster.location )
  end

  def update
    @monster.update( location: params[:button] )

    redirect_to sauron_sauron_monsters_path(@actor), notice: 'Monster moved'
  end

  private

  def set_monster
    @monster = Monster.find(params[:id].to_i)
  end

end
