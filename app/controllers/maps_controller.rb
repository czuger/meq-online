require 'ostruct'

class MapsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor, only: [:edit, :set_influence]
  before_action :set_actor_ensure_board, only: [:show]
  before_action :set_map_data

  MUL_X=1.5442
  MUL_Y=1.5444

  # For board spectators
  def show
    @mul_x = MUL_X
    @mul_y = MUL_Y
  end

  def edit
    @mul_x = MUL_X
    @mul_y = MUL_Y
  end
  
  private

  def set_map_data
    @positions = GameData::Locations.new.position_list
    @influence = @board.influence
    @characters = @board.characters

    @locations = GameData::Locations.new

    @plots = @board.current_plots.order(:plot_position)

    @tokens = @board.get_tokens
  end

end