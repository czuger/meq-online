class SauronMonstersController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor

  def show
    @monsters_list = []

    @board.monsters_on_board.each do |location, monsters|
      monsters.each do |monster|
        @monsters_list << [
            @board.location_name(location),
            @board.monster_name(monster['monster'] ) ]
      end
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
    monster = @game_data_locations_monsters.pick_monster_from_board(@board, location )

    @board.monsters_on_board << monster

    @board.save!

    redirect_to sauron_sauron_monsters_path(@actor)
  end

end
