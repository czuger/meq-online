class LogsController < ApplicationController

  before_action :require_logged_in

  def show
    @heroes = GameData::Heroes.new
    @locations = GameData::Locations.new

    @board = Board.find(params[:board_id])
    @logs = @board.logs.all.includes( { actor: :user } ).order( 'id DESC' )
  end

end
