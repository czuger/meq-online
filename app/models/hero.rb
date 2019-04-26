class Hero < Actor

  belongs_to :user
  belongs_to :board

  has_one :combat

  has_many :movement_preparation_steps, foreign_key: :actor_id

  def deal_damages( damages_amount )
    damages_amount = [ damages_amount, 0 ].max
    life_cards_lost = self.life_pool.shift( damages_amount )
    self.damage_pool += life_cards_lost
    self.damages_taken_this_turn += damages_amount
    self.save!
  end

  #
  # Location methods
  #
  def current_location_name
    @locations ||= GameData::Locations.new
    @locations.get(location).name
  end

  def move_to_regional_haven
    @locations ||= GameData::Locations.new
    region_color = @locations.get(location).color_code
    haven_code = @locations.get_haven_for_color(region_color)
    self.location = haven_code
    self.save!
  end

  #
  # Cards methods
  #
  def card_pic_path(card_number)
    @game_data_heroes ||= GameData::Heroes.new
    @game_data_hero ||= @game_data_heroes.get( name_code )

    @game_data_hero.cards[card_number].pic_path
  end

  def draw_cards( board, nb_cards_to_draw, before_combat= false )
    transaction do
      cards = life_pool.shift(nb_cards_to_draw)
      self.hand += cards
      log_draw_cards!( board, cards.count, before_combat)
      save!
    end
  end

  def rest
    self.life_pool += rest_pool
    self.rest_pool = []
    self.life_pool.shuffle
    save!
  end

  def heal
    transaction do
      rest

      self.life_pool += damage_pool
      self.damage_pool = []
      self.life_pool.shuffle
      save!
    end
  end

  def suffer_peril!(board)
    transaction do
      # current_location_perilous? also instantiate @locations
      if current_location_perilous?(board)
        case Hazard.d4
          when 1
            board.log( self, 'peril.pass_trough', location_name: @locations.get(location).name )
          when 2
            hand_to_life(hand.sample)
            board.log( self, 'peril.lose_card', location_name: @locations.get(location).name )
          when 3
            self.favor -= 1
            board.log( self, 'peril.lose_favor', location_name: @locations.get(location).name )
          when 4
            hand_to_life(hand.sample)
            self.favor -= 1
            board.log( self, 'peril.lose_favor_and_card', location_name: @locations.get(location).name )
          else
            raise 'Hazard is not working, arghhhhh !!!'
        end
      end
      self.save!
    end
  end

  def hand_to_rest(cards)
    discard_cards(cards){ |c| self.rest_pool += c }
  end

  def hand_to_life(cards)
    discard_cards(cards){ |c| self.life_pool += c }
  end

  #
  # Heroes powers
  #
  def argalad_surrounding_monsters
    near_locations = GameData::LocationsPaths.new.get_connected_locations( location )
    board.monsters.where( location: near_locations )
  end

  private

  def current_location_perilous?(board)
    @locations = GameData::Locations.new
    @locations.perilous?(location) || ( board.influence[location] && board.influence[location] > wisdom )
  end

  def cards_from_hand(pool, cards)
    cards = [ cards ] if cards.is_a? Integer
    pool_var = instance_variable_get('@'+pool.to_s+'_pool')
    self.hand -= cards
    pool_var.assign_attributes( pool_var + cards )
  end

  def log_draw_cards!( board, cards_drawn, before_combat= nil )
    action = ''
    action << 'combat.' if before_combat
    action << 'draw_cards'
    board.logs.create!( action: action, params: { name: name_code.to_sym, cards_drawn: cards_drawn, lp_cards: life_pool.count } )
  end

  # Used only with hand_to_rest and hand_to_life
  def discard_cards(cards)
    cards = [] unless cards
    cards = [ cards ] if cards.is_a? Integer

    # We must remove cards one by one, otherwise we would remove too much cards
    # remember that [2, 3, 3, 4] - [2, 3] = [4] and not [3, 4]
    cards.each do |card|
      self.hand.slice!(self.hand.index(card))
    end

    yield(cards)
    self.save!
  end

end
