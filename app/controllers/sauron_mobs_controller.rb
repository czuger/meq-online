class SauronMobsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor
  before_action :set_mob, only: [:edit, :update]

  def index
    @mobs = @board.mobs.order( 'type, updated_at DESC, code' )
  end

  def new
    @available_locations = @board.influence.map{ |k, v| k if v >= 1 }.compact.sort

    black_serpent = @board.mobs.where( code: :black_serpent ).first
    if black_serpent
      surrounding_locations = GameData::LocationsPaths.new.get_connected_locations(black_serpent.location )
      @available_locations += surrounding_locations
    end

    heroes_locations = @board.heroes.map{ |l| l.location }
    @available_locations -= heroes_locations
  end

  def create
    @game_data_locations_monsters = GameData::LocationsMonsters.new

    location = params[:location]

    # If location is crap, this should fail, so it is protected (against injections).
    # TODO : we need to handle an empty monster pool
    @game_data_locations_monsters.place_new_monster(@board, location )

    redirect_to sauron_sauron_mobs_path(@actor)
  end

  def edit
    @available_locations = GameData::LocationsPaths.new.get_connected_locations(@mob.location )

    # Monsters can move only on places that have influence.
    if @mob.is_a? Monster
      @available_locations.reject! { |loc| !( @board.influence[loc] && @board.influence[loc] >= 1 ) }
    end
  end

  def update
    @mob.update( location: params[:button] )

    redirect_to sauron_sauron_mobs_path(@actor), notice: 'Monster moved'
  end

  private

  def set_mob
    @mob = Mob.find(params[:id].to_i)
  end

end
