class SauronMonstersController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor

  def show
  end

  def new
    @available_locations = @board.influence.map{ |k, v| k if v >= 1 }.compact.sort
    heroes_locations = @board.heroes.map{ |l| l.location }
    @available_locations -= heroes_locations
  end

  def place_monster

    redirect_to sauron_sauron_monsters_path(@actor)
  end

end
