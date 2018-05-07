class CombatsController < ApplicationController

  before_action :set_board, only: [:create]
  before_action :set_combat, only: [:show, :update, :destroy, :play_card, :play_card_screen]
  before_action :set_hero, only: [:hero_setup_new, :hero_setup_draw_cards, :hero_setup_increase_strength]

  # GET /combats
  # GET /combats.json
  def index
    @combats = Combat.all
  end

  # GET /combats/1
  # GET /combats/1.json
  def show
    current_user
    if @combat&.hero_choices?
    else
      @fighter_select_data = []

      if @combat&.sauron_user_id == current_user.id
        set_monsters
        @fighter_select_data << [ @monster.name, @combat.monster ]
      end

      if @combat&.hero_user_id == current_user.id
        set_heroes
        @fighter_select_data << [ @heroes_hero.name, @hero.name_code ]
      end
    end

    @should_show_cards = @combat.sauron_card_to_play && @combat.hero_card_to_play

    if @should_show_cards
      set_heroes
      set_monsters
    end

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

  def play_card_screen
    if params[:selected_fighter] == @combat.monster
      set_monsters
    else
      set_heroes
    end
  end

  def play_card
    @player = params[:player]
    if @player == 'sauron'
      set_monsters
      @combat.play_card(@board, true, @monster, params[:selected_card] )
    else
      set_heroes
      @combat.play_card(@board, false, @heroes_hero, params[:selected_card] )
    end
    redirect_to board_combats_path
  end

  # POST /combats
  # POST /combats.json
  def create
    hero = Hero.find( params[:hero_id] )
    monster_code = params[:monster].to_sym
    monsters = GameData::Monsters.new
    monster = monsters.get(monster_code)

    sauron = @board.sauron
    sauron_hand = monster.starting_deck.shift( monster.fortitude )

    cp = { board_id: params[:board_id], hero_id: params[:hero_id], sauron_cards_played:[], hero_cards_played:[],
      monster: monster_code, temporary_strength: hero.strength, sauron_hand: sauron_hand,
      sauron_user_id: sauron.user_id, hero_user_id: hero.user_id }

    @combat = Combat.new(cp)

    respond_to do |format|
      if @combat.save
        format.html { redirect_to board_combats_path( params[:board_id] ), notice: 'Combat was successfully created.' }
      else
        set_new_data
        format.html { render :new }
      end
    end
  end

  def hero_setup_new
    set_heroes
  end

  def hero_setup_draw_cards
    @combat.transaction do
      @hero.draw_cards( @board, params[:nb_cards_to_draw].to_i, true )
      @combat.start!
    end
    redirect_to board_combats_path( @board )
  end

  def hero_setup_increase_strength
    @combat.transaction do
      @combat.update( temporary_strength: params[:increase_strength] )
      @combat.log_increase_strength!( @board, @combat, @hero )
      @combat.start!
    end
    redirect_to board_combats_path( @board )
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
      format.html { redirect_to boards_path, notice: 'Combat was successfully destroyed.' }
      format.json { head :no_content }
    end
  end

  private
    # Use callbacks to share common setup or constraints between actions.

    def set_board
      @board ||= Board.find(params[:board_id])
    end

    def set_combat
      set_board
      @combat ||= @board.combat
    end

    def set_hero
      set_combat
      @hero ||= @combat.hero
    end

    def set_heroes
      set_hero
      @heroes = GameData::Heroes.new
      @heroes_hero = @heroes.get(@hero.name_code)
    end

    def set_monsters
      set_combat
      @monsters = GameData::Monsters.new
      @monster = @monsters.get(@combat.monster.to_sym)
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
