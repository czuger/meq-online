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

    @quests_positions = []
    %w( plot-card-1 plot-card-2 plot-card-3 ).each do |plot|
      if @board.current_plots[plot]
        @quest = @board.current_plots[plot]

      end

    end

  end

end