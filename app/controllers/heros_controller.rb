class HerosController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor

  def index
    @board = Board.find(params[:board_id])
    @heros = @board.heroes.select{ |h| h.user_id == current_user.id }
  end

  def show
    if @actor.active
      @heroes = GameData::Heroes.new
      @heroes_hero = @heroes.get( @actor.name_code )

      @locations = GameData::Locations.new
      @locations.delete!(@actor.location)

      @cards = @actor.hand
    else
      redirect_to board_inactive_actor_path(@board)
    end
  end

  def draw_cards
    nb_cards_to_draw = params[:nb_cards].to_i
    cards = @actor.life_pool.shift(nb_cards_to_draw)
    @actor.hand += cards
    @actor.save!

    @actor.log_draw_cards!( @board, cards.count)

    redirect_to @actor
  end

  def rest
    @actor.life_pool += @actor.rest_pool
    @actor.rest_pool = []
    @actor.life_pool.shuffle
    @actor.save!
    redirect_to @actor
  end

  def heal
    @actor.life_pool += @actor.damage_pool
    @actor.damage_pool = []
    @actor.life_pool.shuffle
    @actor.save!
    redirect_to @actor
  end

  def take_damages
    damage_amount = params[:damage_amount].to_i
    damages_taken_from_life_pool = @actor.life_pool.shift(damage_amount)
    @actor.damage_pool += damages_taken_from_life_pool
    rest_damages = damage_amount - damages_taken_from_life_pool.count
    @actor.damage_pool += @actor.hand.shift(rest_damages)
    @actor.save!
    redirect_to @actor
  end

  def finish_turn
    @actor.transaction do
      @actor.update( turn_finished: true )
      @board.log!( current_user, @actor, :finish_turn )
    end
    redirect_to @actor
  end

  def move
    hero_location = @actor.location
    @actor.location = params['move_to']
    card = params['card_used'].to_i

    card_position = @actor.hand.index( card )
    if card_position
      @actor.hand.delete_at( card_position )

      @actor.rest_pool << card
      @actor.save!

      @actor.log_movement!( @board, card )
    else
      raise "Can't find a card position. card = #{card.inspect}, hand = #{@actor.hand.inspect}"
    end

    redirect_to @actor
  end

end
