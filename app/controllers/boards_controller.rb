class BoardsController < ApplicationController

  before_action :require_logged_in, except: [:welcome]
  before_action :set_board, except: [:index, :new, :create, :welcome]
  before_action :set_sauron_ensure_sauron, only: [:map], except: [:story_screen, :welcome]

  def welcome
  end

  # GET /boards
  # GET /boards.json
  def index
    if params[:all]
      @boards = Board.all.includes( :heroes, :sauron, { heroes: :user }, { sauron: :user } ).order( 'updated_at DESC' )
    else
      @boards = @current_user.boards.includes( :heroes, :sauron, { heroes: :user }, { sauron: :user } ).order( 'updated_at DESC' )
    end

    @heroes = GameData::Heroes.new
  end

  # GET /boards/1
  # GET /boards/1.json

  def inactive_actor
  end

  # GET /boards/new
  def new
    @board = Board.new
    @remaining_heroes = @board.max_heroes_count
    @sauron_state = true
    load_heroes
  end

  # # GET /boards/1/edit
  # def edit
  # end

  def join_new
    @remaining_heroes = [ @board.max_heroes_count - @board.current_heroes_count, @board.max_heroes_count ].min
    load_heroes
    @sauron_state = ( @current_user.id == @board.sauron&.user_id )
    @sauron_disabled = @board.sauron
  end

  def join
    add_players_to_board

    redirect_to boards_path
  end


  # POST /boards
  # POST /boards.json
  def create

    @board, starting_plot, starting_plot_id = Board.create_new_board

    respond_to do |format|
      @board.transaction do
        if @board.save
          add_players_to_board

          # GameData::LocationsMonsters.new.fill_board(@board)

          @board.create_monster( :mouth_of_sauron, :dol_guldur )
          @board.create_monster( :black_serpent, :near_harad )

          @board.log( nil, 'setup.set_starting_plot', { plot_card: starting_plot_id } )
          @board.log( nil, 'setup.set_influence_in_shadow_pool', { count: starting_plot.influence.shadow_pool } )

          starting_plot.influence.init.each do |key, value|
            @board.log( nil, 'setup.set_influence_in_location', { count: value, location: key.to_s.humanize } )
          end

          @board.set_plot nil, 1, starting_plot_id, starting_plot

          format.html { redirect_to boards_path, notice: 'Board was successfully created.' }
          format.json { render :show, status: :created, location: @board }
        else
          format.html { render :new }
          format.json { render json: @board.errors, status: :unprocessable_entity }
        end
      end
    end
  end

  def story_screen
    @story_data = @board.story_data
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
      @heroes = GameData::Heroes.new
      existing_heroes = @board.heroes.pluck(:name_code).map{ |h| h.to_sym }

      # For now, we forbidd :beravor, :eleanor, :thalin
      # existing_heroes += [ :beravor, :eleanor, :thalin ]

      @heroes.delete_heroes!( existing_heroes )
    end


    def add_players_to_board
      load_heroes

      # Adding heroes
      @board.transaction do
        # heroes_to_process = [:hero_1, :hero_2, :hero_3].map{ |h| params[h]&.to_sym }.compact.reject{ |h| h.empty? }.uniq
        # heroes_to_process = [:hero_1, :hero_2, :hero_3, :]
        heroes_to_process = [:beravor, :eleanor, :thalin, :eometh]
        heroes_to_process.each_with_index do |hero_code, index|

          ai_user = User.create_ia_user(index)
          hero = @heroes.get( hero_code )

          life_pool = hero.starting_deck.shuffle
          # hand = life_pool.shift( hero[:fortitude] )
          hand = []

          quests_manager = GameData::HeroQuests.new( 'regular' )
          starting_quest = quests_manager.get_starting_quest( hero_code )
          quests_manager.hero_setup( @board, hero_code )

          items = {}
          items[:horse] = true if hero_code == :eometh

          @board.heroes.create!(
              name_code: hero_code, fortitude: hero[:fortitude], strength: hero[:strength], agility: hero[:agility],
              wisdom: hero[:wisdom], location: hero[:start_location_code_name], life_pool: life_pool,
              rest_pool: [], damage_pool: [], hand: hand, user_id: ai_user.id, name: hero.name,
              current_quest: starting_quest, playing_order: index, items: items, used_powers: []
          )
          # Just tell that the user is connected to this board
          @current_user.boards << @board unless @current_user.boards.include?( @board )
        end

        # Adding Sauron
        @board.create_sauron!( plot_cards: [], shadow_cards: [], drawn_plot_cards: [], drawn_shadow_cards: [],
                               user_id: @current_user.id, name: 'Sauron' )
        # Just tell that the user is connected to this board
        @current_user.boards << @board unless @current_user.boards.include?( @board )

        # Misc
        @board.current_heroes_count= @board.heroes.count
        @board.sauron_created= @board.sauron ? true : false

        @board.next_to_sauron_setup_screen!

        @board.save!
      end
    end
end