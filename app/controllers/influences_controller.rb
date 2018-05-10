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

    @board.transaction do

      diff_hash = Hash[ tmp_hash.to_a.sort - @board.influence.to_a.sort ]
      diff_hash.each do |k, v|
        @board.logs.create!( action: :place_influence, params:{ place: @locations.get(k).name, value: v },
                             user_id: current_user.id, actor_id: @actor_id)
      end

      @board.influence.merge!( tmp_hash )
      @board.save!
    end

    redirect_to board_actor_influences_path(@board,@actor_id)
  end

  def show
    @locations= GameData::Locations.new.list_by_region
    @influence = @board.influence
  end

  private

  def set_board
    @board = Board.find(params[:board_id])
    @actor_id = params[:actor_id].to_i
    raise "Board #{@board.inspect} can't be modified by #{current_user.id}" unless @board.users.pluck(:id).include?( current_user.id )
  end

end