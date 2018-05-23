class InfluencesController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_board, only: [:show, :edit, :update]
  before_action :set_influence, only: [:show, :edit]

  def edit
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
                             user_id: current_user.id, actor: @actor)
      end

      @board.influence.merge!( tmp_hash )
      @board.save!
    end

    redirect_to influence_path(@actor) if params[:redirect] != 'false'
  end

  def show
  end

  private

  def set_influence
    @locations= GameData::Locations.new.list_by_region
    @influence = @board.influence
  end

end
