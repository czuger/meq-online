class SauronsController < ApplicationController
  before_action :require_logged_in
  before_action :set_sauron, only: [:show, :edit, :update, :destroy]

  # # GET /saurons
  # # GET /saurons.json
  # def index
  #   @saurons = Sauron.all
  # end
  #
  # GET /saurons/1
  # GET /saurons/1.json
  def show
    @player = @sauron
  end
  #
  # # GET /saurons/new
  # def new
  #   @sauron = Sauron.new
  # end
  #
  # # GET /saurons/1/edit
  # def edit
  # end
  #
  # # POST /saurons
  # # POST /saurons.json
  # def create
  #   @sauron = Sauron.new(sauron_params)
  #
  #   respond_to do |format|
  #     if @sauron.save
  #       format.html { redirect_to @sauron, notice: 'Sauron was successfully created.' }
  #       format.json { render :show, status: :created, location: @sauron }
  #     else
  #       format.html { render :new }
  #       format.json { render json: @sauron.errors, status: :unprocessable_entity }
  #     end
  #   end
  # end
  #
  # # PATCH/PUT /saurons/1
  # # PATCH/PUT /saurons/1.json
  # def update
  #   respond_to do |format|
  #     if @sauron.update(sauron_params)
  #       format.html { redirect_to @sauron, notice: 'Sauron was successfully updated.' }
  #       format.json { render :show, status: :ok, location: @sauron }
  #     else
  #       format.html { render :edit }
  #       format.json { render json: @sauron.errors, status: :unprocessable_entity }
  #     end
  #   end
  # end
  #
  # # DELETE /saurons/1
  # # DELETE /saurons/1.json
  # def destroy
  #   @sauron.destroy
  #   respond_to do |format|
  #     format.html { redirect_to saurons_url, notice: 'Sauron was successfully destroyed.' }
  #     format.json { head :no_content }
  #   end
  # end

  private
    # Use callbacks to share common setup or constraints between actions.
    def set_sauron
      @board = Board.find(params[:board_id])
      @sauron = @board.sauron
    end

    # Never trust parameters from the scary internet, only allow the white list through.
    def sauron_params
      params.require(:sauron).permit(:board_id, :user_id, :plot_cards, :shadow_cards)
    end
end
