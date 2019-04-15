class CombatsController < ApplicationController

  before_action :require_logged_in
  before_action :set_combat
  before_action :set_actor_ensure_actor, only: []

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

  def play_combat_card
  end

  def hero_setup_new
  end

  def hero_setup
    temporary_strength = @hero.strength

    @combat.transaction do
      if params[:button] == 'draw'
        @hero.draw_cards( @board, @hero.agility, true )
      else
        temporary_strength += @hero.agility

        @board.log( @hero, 'combat.inc_strength',name: @hero.name_code, str: @hero.strength,
                    nstr: temporary_strength )
      end

      @combat.temporary_hero_strength = temporary_strength
      @combat.save!
      
      @board.next_to_play_combat_card!
      redirect_to play_combat_card_screen_board_combats_path( @board, @hero )
    end
  end

  # PATCH/PUT /combats/1
  # PATCH/PUT /combats/1.json
  # def update
  #   respond_to do |format|
  #     if @combat.update(combat_params)
  #       format.html { redirect_to @combat, notice: 'Combat was successfully updated.' }
  #       format.json { render :show, status: :ok, location: @combat }
  #     else
  #       format.html { render :edit }
  #       format.json { render json: @combat.errors, status: :unprocessable_entity }
  #     end
  #   end
  # end

  private
    # Use callbacks to share common setup or constraints between actions.
    def set_combat
      @board = Board.find(params[:board_id])
      @combat = @board.combat
      @hero = @combat.hero
      @actor = @hero
      @mob = @combat.mob
    end

    # Never trust parameters from the scary internet, only allow the white list through.
    def combat_params
      params.permit(:board_id, :hero_id )
    end

    def set_new_data
      @board = Board.find( params[:board_id] )
      @heroes = GameData::Heroes.new
      @monsters = GameData::Mobs.new

      @heroes_select_array = @board.heroes.map{ |h| [ @heroes.get( h.name_code ).name, h.id ] }
    end
end
