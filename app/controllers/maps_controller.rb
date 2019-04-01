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

    @tokens = {}
    @board.characters.each do |character, location|
      @tokens[location] ||= []
      @tokens[location] << "characters/small_#{character}.jpg".freeze
    end
    @board.current_plots.each do |plot|
      @tokens[plot.affected_location] ||= []
      @tokens[plot.affected_location] << 'quest_1.png'.freeze
    end
    @board.favors.each do |favor_location|
      @tokens[favor_location] ||= []
      @tokens[favor_location] << 'favor.png'.freeze
    end

    pp @tokens

  end

end