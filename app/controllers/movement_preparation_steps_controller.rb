class MovementPreparationStepsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor
  before_action :set_heroes_hero_and_locations, only: [:edit, :new]
  before_action :set_movement_preparation_step, only: [:show, :edit, :update, :destroy]

  # GET /movement_preparation_steps
  # GET /movement_preparation_steps.json
  def index
    @movement_preparation_steps = @hero.movement_preparation_steps
  end

  # GET /movement_preparation_steps/1
  # GET /movement_preparation_steps/1.json
  def show
  end

  # GET /movement_preparation_steps/new
  def new
    @movement_preparation_step = MovementPreparationStep.new
    @heroes_hero = @heroes.get( @actor.name_code )
  end

  # GET /movement_preparation_steps/1/edit
  def edit
  end

  # POST /movement_preparation_steps
  # POST /movement_preparation_steps.json
  def create
    @movement_preparation_step = MovementPreparationStep.new(movement_preparation_step_params)

    respond_to do |format|
      if @movement_preparation_step.save
        format.html { redirect_to @movement_preparation_step, notice: 'Movement preparation step was successfully created.' }
        format.json { render :show, status: :created, location: @movement_preparation_step }
      else
        format.html { render :new }
        format.json { render json: @movement_preparation_step.errors, status: :unprocessable_entity }
      end
    end
  end

  # PATCH/PUT /movement_preparation_steps/1
  # PATCH/PUT /movement_preparation_steps/1.json
  def update
    respond_to do |format|
      if @movement_preparation_step.update(movement_preparation_step_params)
        format.html { redirect_to @movement_preparation_step, notice: 'Movement preparation step was successfully updated.' }
        format.json { render :show, status: :ok, location: @movement_preparation_step }
      else
        format.html { render :edit }
        format.json { render json: @movement_preparation_step.errors, status: :unprocessable_entity }
      end
    end
  end

  # DELETE /movement_preparation_steps/1
  # DELETE /movement_preparation_steps/1.json
  def destroy
    @movement_preparation_step.destroy
    respond_to do |format|
      format.html { redirect_to movement_preparation_steps_url, notice: 'Movement preparation step was successfully destroyed.' }
      format.json { head :no_content }
    end
  end

  private
    # Use callbacks to share common setup or constraints between actions.
    def set_movement_preparation_step
      @movement_preparation_step = @actor.movement_preparation_steps.find(params[:id])
    end

    def set_heroes_hero_and_locations
      @heroes = GameData::Heroes.new
      @heroes_hero = @heroes.get( @actor.name_code )

      @locations = GameData::Locations.new
      @locations.delete!(@actor.location)
    end

    # Never trust parameters from the scary internet, only allow the white list through.
    def movement_preparation_step_params
      params.require(:movement_preparation_step).permit(:board_id, :hero_id, :origine, :destination, :card_used, :order, :validation_required)
    end
end
