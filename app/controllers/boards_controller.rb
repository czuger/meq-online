class BoardsController < ApplicationController
  before_action :set_board, only: [:show, :edit, :update, :destroy]

  # GET /boards
  # GET /boards.json
  def index
    @boards = Board.all
  end

  # GET /boards/1
  # GET /boards/1.json
  def show
  end

  # GET /boards/new
  def new
    load_heroes
    @board = Board.new
  end

  # GET /boards/1/edit
  def edit
  end

  # POST /boards
  # POST /boards.json
  def create

    @board = Board.new

    respond_to do |format|
      if @board.save
        load_heroes

        [:hero_1, :hero_2, :hero_3].each do |hero|
          unless params[hero].empty?
            hero_code = params[hero].to_sym
            hero = @heroes[hero_code]
            load_hero_starting_deck(hero_code)
            @board.heroes.create!(
              name_code: hero_code, fortitude: hero[:fortitude], strength: hero[:strength], agility: hero[:agility],
              wisdom: hero[:wisdom], location: hero[:start_location_code_name], life_pool: @starting_deck.shuffle,
              rest_pool: [], damage_pool: []
            )
          end
        end


        format.html { redirect_to @board, notice: 'Board was successfully created.' }
        format.json { render :show, status: :created, location: @board }
      else
        format.html { render :new }
        format.json { render json: @board.errors, status: :unprocessable_entity }
      end
    end
  end

  # PATCH/PUT /boards/1
  # PATCH/PUT /boards/1.json
  def update
    respond_to do |format|
      if @board.update(board_params)
        format.html { redirect_to @board, notice: 'Board was successfully updated.' }
        format.json { render :show, status: :ok, location: @board }
      else
        format.html { render :edit }
        format.json { render json: @board.errors, status: :unprocessable_entity }
      end
    end
  end

  # DELETE /boards/1
  # DELETE /boards/1.json
  def destroy
    @board.destroy
    respond_to do |format|
      format.html { redirect_to boards_url, notice: 'Board was successfully destroyed.' }
      format.json { head :no_content }
    end
  end

  private
    # Use callbacks to share common setup or constraints between actions.
    def set_board
      @board = Board.find(params[:id])
    end

    # Never trust parameters from the scary internet, only allow the white list through.
    def board_params
      params.require(:board).permit(:heroes)
    end

    def load_heroes
      @heroes = YAML.load_file('app/models/data/heroes/heroes_list.yaml')
    end

    def load_hero_starting_deck( hero_name_code )
      fname = "app/models/data/heroes/#{hero_name_code}_starting_deck.yaml"
      puts "About to load #{fname}"
      @starting_deck = YAML.load_file(fname)
    end
end