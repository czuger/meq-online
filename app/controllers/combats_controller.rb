class CombatsController < ApplicationController
  before_action :set_combat, only: [:show, :edit, :update, :destroy]

  # GET /combats
  # GET /combats.json
  def index
    @combats = Combat.all
  end

  # GET /combats/1
  # GET /combats/1.json
  def show
  end

  # GET /combats/new

  # Only one combat per board
  # Combat step :
  # - creation
  # - setup : hero player choose if he use fortitude and draw cards
  # - Play card step
  #   - each player choose a card
  # - Card result
  #   - each player mark the result (take damages, discard card, show next card to opponent)
  # - Next turn

  def new
    set_new_data
    @combat = Combat.new( board: @board )
  end

  # GET /combats/1/edit
  def edit
  end

  # POST /combats
  # POST /combats.json
  def create

    @hero = Hero.find( params[:hero_id] )

    cp = { board_id: params[:board_id], hero_id: params[:hero_id], sauron_cards_played:[], hero_cards_played:[],
      monster: params[:monster], temporary_strength: @hero.strength }

    @combat = Combat.new(cp)

    respond_to do |format|
      if @combat.save
        format.html { redirect_to @combat, notice: 'Combat was successfully created.' }
      else
        set_new_data
        format.html { render :new }
      end
    end
  end

  # PATCH/PUT /combats/1
  # PATCH/PUT /combats/1.json
  def update
    respond_to do |format|
      if @combat.update(combat_params)
        format.html { redirect_to @combat, notice: 'Combat was successfully updated.' }
        format.json { render :show, status: :ok, location: @combat }
      else
        format.html { render :edit }
        format.json { render json: @combat.errors, status: :unprocessable_entity }
      end
    end
  end

  # DELETE /combats/1
  # DELETE /combats/1.json
  def destroy
    @combat.destroy
    respond_to do |format|
      format.html { redirect_to combats_url, notice: 'Combat was successfully destroyed.' }
      format.json { head :no_content }
    end
  end

  private
    # Use callbacks to share common setup or constraints between actions.
    def set_combat
      @combat = Combat.find(params[:id])
    end

    # Never trust parameters from the scary internet, only allow the white list through.
    def combat_params
      params.permit(:board_id, :hero_id )
    end

    def set_new_data
      @board = Board.find( params[:board_id] )
      @heroes = GameData::Heroes.new
      @monsters = GameData::Monsters.new

      @heroes_select_array = @board.heroes.map{ |h| [ @heroes.get( h.name_code ).name, h.id ] }
    end
end
