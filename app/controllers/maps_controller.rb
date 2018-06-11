class MapsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor, only: [:edit]
  before_action :set_actor_ensure_board, only: [:show]

  # For board spectators
  def show
  end

  def edit
    @positions = GameData::Locations.new.position_list
    @influence = @board.influence
  end

end