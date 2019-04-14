class SauronMonstersController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor

  def show
    @monsters_list = []

    @board.monsters.each do |monster|
      @monsters_list << [
          @board.location_name(monster.location),
          @board.monster_name(monster.code ) ]
    end

    @monsters_list.sort!
  end

  def new
    @available_locations = @board.influence.map{ |k, v| k if v >= 1 }.compact.sort
    heroes_locations = @board.heroes.map{ |l| l.location }
    @available_locations -= heroes_locations
  end

  def place_monster
    @game_data_locations_monsters = GameData::LocationsMonsters.new

    location = params[:location]

    # If location is crap, this should fail, so it is protected (against injections).
    # TODO : we need to handle an empty monster pool
    @game_data_locations_monsters.place_new_monster(@board, location )

    redirect_to sauron_sauron_monsters_path(@actor)
  end

end
