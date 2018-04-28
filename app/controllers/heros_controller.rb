class HerosController < ApplicationController
  before_action :set_hero, only: [:show, :edit, :update, :destroy, :draw_cards, :rest, :heal, :take_damages]

  # GET /heros
  # GET /heros.json
  def index
    @board = Board.find(params[:board_id])
    @heros = @board.heroes.all
  end

  # GET /heros/1
  # GET /heros/1.json
  def show
    @hero_cards = YAML.load_file("app/models/data/heroes/#{@hero.name_code}_actions_cards.yaml")
  end

  # GET /heros/new
  def new
    @hero = Hero.new
  end

  # GET /heros/1/edit
  def edit
  end

  def draw_cards
    @hero.hand += @hero.life_pool.shift(@hero.fortitude)
    @hero.save!
    redirect_to [@board,@hero]
  end

  def rest
    @hero.life_pool += @hero.rest_pool
    @hero.rest_pool = []
    @hero.life_pool.shuffle
    @hero.save!
    redirect_to [@board,@hero]
  end

  def heal
    @hero.life_pool += @hero.damage_pool
    @hero.damage_pool = []
    @hero.life_pool.shuffle
    @hero.save!
    redirect_to [@board,@hero]
  end

  def take_damages
    damage_amount = params[:damage_amount].to_i
    damages_taken_from_life_pool = @hero.life_pool.shift(damage_amount)
    @hero.damage_pool += damages_taken_from_life_pool
    rest_damages = damage_amount - damages_taken_from_life_pool.count
    @hero.damage_pool += @hero.hand.shift(rest_damages)
    @hero.save!
    redirect_to [@board,@hero]
  end

  def move_edit

  end

  # POST /heros
  # POST /heros.json
  def create
    @hero = Hero.new(hero_params)

    respond_to do |format|
      if @hero.save
        format.html { redirect_to @hero, notice: 'Hero was successfully created.' }
        format.json { render :show, status: :created, location: @hero }
      else
        format.html { render :new }
        format.json { render json: @hero.errors, status: :unprocessable_entity }
      end
    end
  end

  # PATCH/PUT /heros/1
  # PATCH/PUT /heros/1.json
  def update
    respond_to do |format|
      if @hero.update(hero_params)
        format.html { redirect_to @hero, notice: 'Hero was successfully updated.' }
        format.json { render :show, status: :ok, location: @hero }
      else
        format.html { render :edit }
        format.json { render json: @hero.errors, status: :unprocessable_entity }
      end
    end
  end

  # DELETE /heros/1
  # DELETE /heros/1.json
  def destroy
    @hero.destroy
    respond_to do |format|
      format.html { redirect_to heros_url, notice: 'Hero was successfully destroyed.' }
      format.json { head :no_content }
    end
  end

  private
    # Use callbacks to share common setup or constraints between actions.
    def set_hero
      @board = Board.find(params[:board_id])
      @hero = @board.heroes.find(params[:id]||params[:hero_id])
    end

    # Never trust parameters from the scary internet, only allow the white list through.
    def hero_params
      params.require(:hero).permit(:name_code, :fortitude, :strength, :agility, :wisdom, :location, :life_pool, :rest_pool, :damage_pool)
    end
end
