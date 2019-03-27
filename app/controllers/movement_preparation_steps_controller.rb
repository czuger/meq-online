class MovementPreparationStepsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor
  before_action :set_heroes_hero_and_locations, only: [:edit, :new, :create, :index]
  before_action :set_remaining_cards, only: [:edit, :new]
  before_action :set_movement_preparation_step, only: [:edit, :update, :destroy]

  # GET /movement_preparation_steps
  # GET /movement_preparation_steps.json
  def index
    @movement_preparation_steps = @actor.movement_preparation_steps
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
    params = movement_preparation_step_params.merge(origine: @last_location)
    @movement_preparation_step = @actor.movement_preparation_steps.new(params)

    respond_to do |format|
      if @movement_preparation_step.save
        format.html { redirect_to hero_movement_preparation_steps_path(@actor), notice: 'Movement preparation step was successfully created.' }
      else
        format.html { render :new }
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
      format.html { redirect_to hero_movement_preparation_steps_path (@actor), notice: 'Movement preparation step was successfully destroyed.' }
    end
  end

  def terminate
    @board.transaction do
      @board.next_to_movement_break_schedule!
      @board.switch_to_sauron
      @board.save!

      redirect_to boards_path
    end
  end

  private
    # Use callbacks to share common setup or constraints between actions.
    def set_movement_preparation_step
      @movement_preparation_step = @actor.movement_preparation_steps.find(params[:id])
    end

    def set_remaining_cards
      @cards = @actor.hand
      used_cards = @actor.movement_preparation_steps.pluck(:selected_cards).each do |cards_set|
        cards_set.each do |card|
          first_card_position = @cards.index( card )
          @cards.delete_at( first_card_position )
        end
      end
    end

    def set_heroes_hero_and_locations
      @heroes = GameData::Heroes.new
      @heroes_hero = @heroes.get( @actor.name_code )

      last_movement_preparation_step = @actor.movement_preparation_steps.last
      if last_movement_preparation_step
        @last_location = last_movement_preparation_step.destination
      else
        @last_location = @actor.location
      end

      @locations = GameData::LocationsPaths.new.get_connected_locations_for_select(@last_location)
      # @locations.delete!(@actor.location)

      @selectable_card_class = 'selectable-card-selection-multiple'
    end

    # Never trust parameters from the scary internet, only allow the white list through.
    def movement_preparation_step_params
      p = params.require(:movement_preparation_step).permit(:destination, :selected_cards)
      p[:selected_cards] = p[:selected_cards].split( ',' ).map(&:to_i)
      p
    end
end
