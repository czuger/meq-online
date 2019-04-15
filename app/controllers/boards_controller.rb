class BoardsController < ApplicationController

  before_action :require_logged_in
  before_action :set_board, except: [:index, :new, :create]
  before_action :set_actor_ensure_actor, only: [:map], except: [:story_screen]

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
    starting_plot_id= rand( 0..2 )
    starting_plot = GameData::Plots.new.get(starting_plot_id)

    plot_deck= ((3..17).to_a - [10, 11, 14, 15, 17]).to_a.shuffle

    # We currently remove shadow cards that are played during hero movement.
    shadow_deck= ((0..23).to_a - [1, 21]).shuffle
    event_deck = GameData::Events.new.get_starting_event_deck

    max_heroes_count= params[:max_heroes_count].to_i

    @board = Board.new(
        influence: starting_plot.influence.init,
        plot_deck: plot_deck,
        shadow_deck: shadow_deck,
        event_deck: event_deck,
        plot_discard: [],
        shadow_discard: [],
        max_heroes_count: max_heroes_count,
        shadow_pool: starting_plot.influence.shadow_pool,
        characters: {}
    )

    respond_to do |format|
      @board.transaction do
        if @board.save
          add_players_to_board

          GameData::LocationsMonsters.new.fill_board(@board)

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
    @heroes_to_final = 18 - @board.story_marker_heroes

    sauron_highest_marker = [ @board.story_marker_ring, @board.story_marker_conquest, @board.story_marker_corruption ].max
    @sauron_to_final = 18 - sauron_highest_marker

    shadowfall_points = 0
    %w( story_marker_ring story_marker_conquest story_marker_corruption ).each do |sm|
      shadowfall_points += [ @board.send( sm ), 10 ].min
    end
    @sauron_to_shadowfall = 30 - shadowfall_points

    @sauron_dominance = [ @sauron_to_final, @sauron_to_shadowfall ].min

    if @heroes_to_final < @sauron_dominance
      @dominance = :heroes
    elsif @heroes_to_final > @sauron_dominance
      @dominance = :sauron
    else
      @dominance = :nobody
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
      @heroes = GameData::Heroes.new
      existing_heroes = @board.heroes.pluck(:name_code).map{ |h| h.to_sym }
      @heroes.delete_heroes!( existing_heroes )
    end


    def add_players_to_board
      load_heroes

      # Adding heroes
      @board.transaction do
        heroes_to_process = [:hero_1, :hero_2, :hero_3].map{ |h| params[h]&.to_sym }.compact.reject{ |h| h.empty? }.uniq
        heroes_to_process.each_with_index do |hero_code, index|

          hero = @heroes.get( hero_code )

          life_pool = hero.starting_deck.shuffle
          # hand = life_pool.shift( hero[:fortitude] )
          hand = []

          quests_manager = GameData::HeroQuests.new( 'regular' )
          starting_quest = quests_manager.get_starting_quest( hero_code )
          quests_manager.hero_setup( @board, hero_code )

          @board.heroes.create!(
              name_code: hero_code, fortitude: hero[:fortitude], strength: hero[:strength], agility: hero[:agility],
              wisdom: hero[:wisdom], location: hero[:start_location_code_name], life_pool: life_pool,
              rest_pool: [], damage_pool: [], hand: hand, user_id: @current_user.id, name: hero.name,
              current_quest: starting_quest, playing_order: index
          )
          # Just tell that the user is connected to this board
          @current_user.boards << @board unless @current_user.boards.include?( @board )
        end

        # Adding Sauron
        if params[:sauron]
          @board.create_sauron!( plot_cards: [], shadow_cards: [], drawn_plot_cards: [], drawn_shadow_cards: [], user_id: @current_user.id )

          # Just tell that the user is connected to this board
          @current_user.boards << @board unless @current_user.boards.include?( @board )
        end

        # Misc
        @board.current_heroes_count= @board.heroes.count
        @board.sauron_created= @board.sauron ? true : false

        if @board.current_heroes_count < @board.max_heroes_count || !@board.sauron_created
          @board.wait_for_players! unless @board.waiting_for_players?
        else
          # unless @board.sauron_turn?

            @board.set_sauron_activation_state( true )
            @board.next_to_sauron_setup!
          # end
        end

        @board.save!
      end
    end
end