class BoardsController < ApplicationController

  before_action :require_logged_in
  before_action :set_board, only: [:show, :edit, :update, :destroy, :join_new, :join]

  # GET /boards
  # GET /boards.json
  def index
    if params[:all]
      @boards = Board.all.includes( :heroes, :sauron, { heroes: :user }, { sauron: :user } )
    else
      @boards = @current_user.boards.includes( :heroes, :sauron, { heroes: :user }, { sauron: :user } )
    end

  end

  # GET /boards/1
  # GET /boards/1.json
  def show
  end

  # GET /boards/new
  def new
    @board = Board.new
    load_heroes
  end

  # GET /boards/1/edit
  def edit
  end

  def join_new
    load_heroes
    @sauron_state = ( @board.sauron.user_id == @current_user.id )
    @sauron_disabled = @board.sauron
  end

  def join
    add_players_to_board
    redirect_to boards_path
  end


  # POST /boards
  # POST /boards.json
  def create

    @board = Board.new
    @board.max_heroes_count= params[:max_heroes_count]

    respond_to do |format|
      @board.transaction do
        if @board.save
          add_players_to_board

          format.html { redirect_to boards_path, notice: 'Board was successfully created.' }
          format.json { render :show, status: :created, location: @board }
        else
          format.html { render :new }
          format.json { render json: @board.errors, status: :unprocessable_entity }
        end
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
    @board.users.clear
    @board.destroy
    respond_to do |format|
      format.html { redirect_to boards_url, notice: 'Board was successfully destroyed.' }
      format.json { head :no_content }
    end
  end

  private
    # Use callbacks to share common setup or constraints between actions.
    def set_board
      @board = Board.find(params[:id]||params[:board_id])
    end

    # Never trust parameters from the scary internet, only allow the white list through.
    def board_params
      params.require(:board).permit(:heroes)
    end

    def load_heroes
      @heroes = YAML.load_file('app/models/data/heroes/heroes_list.yaml')
      existing_heroes = @board.heroes.pluck(:name_code).map{ |h| h.to_sym }
      @heroes.reject!{ |k, v| existing_heroes.include?( k ) }
    end

    def load_hero_starting_deck( hero_name_code )
      fname = "app/models/data/heroes/#{hero_name_code}_starting_deck.yaml"
      puts "About to load #{fname}"
      @starting_deck = YAML.load_file(fname)
    end

    def add_players_to_board
      load_heroes

      @board.transaction do
        heroes_to_process = [:hero_1, :hero_2, :hero_3].map{ |h| params[h].to_sym }.reject{ |h| h.empty? }.uniq
        heroes_to_process.each do |hero_code|

          hero = @heroes[hero_code]
          load_hero_starting_deck(hero_code)

          @board.heroes.create!(
              name_code: hero_code, fortitude: hero[:fortitude], strength: hero[:strength], agility: hero[:agility],
              wisdom: hero[:wisdom], location: hero[:start_location_code_name], life_pool: @starting_deck.shuffle,
              rest_pool: [], damage_pool: [], hand: [], user_id: @current_user.id
          )
          @current_user.boards << @board unless @current_user.boards.include?( @board )
        end

        if params[:sauron]
          @board.create_sauron!( plot_cards: [], shadow_cards: [], user_id: @current_user.id )
          @current_user.boards << @board unless @current_user.boards.include?( @board )
        end

        @board.current_heroes_count= @board.heroes.count
        @board.sauron_created= @board.sauron ? true : false
        @board.save!
      end
    end
end