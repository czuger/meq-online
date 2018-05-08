class InfluencesController < ApplicationController

  before_action :require_logged_in
  before_action :set_board, only: [:show, :edit, :update]

  def edit
    @locations= GameData::Locations.new.list_by_region
    @influence = @board.influence
  end

  def update
    ActionController::Parameters.permit_all_parameters = true
    @locations= GameData::Locations.new
    tmp_hash = params[:locations].select{ |l, v| !v.empty? && @locations.exist?( l ) }.to_h
    tmp_hash.transform_values!{ |v| v.to_i }
    tmp_hash = tmp_hash.symbolize_keys
    @board.influence.merge!( tmp_hash )
    @board.save!
    redirect_to board_influences_path(@board)
  end

  def show
    @locations= GameData::Locations.new.list_by_region
    @influence = @board.influence
  end

  private

  def set_board
    @board = Board.find(params[:board_id])
    raise "Board #{@board.inspect} can't be modified by #{current_user.id}" unless @board.users.pluck(:id).include?( current_user.id )
  end

end
