require 'ostruct'

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
    @characters = @board.characters

    @locations = GameData::Locations.new

    @plots = @board.current_plots.order(:plot_position)

    @tokens = GameEngine::DataAtLocation.new.gather(@board).tokens

    @scale = params[:scale] || 'x1'
  end

end