class Hero < Actor

  include GameEngine::HeroDamages
  include GameEngine::HeroPeril
  include GameEngine::HeroCorruption

  belongs_to :user
  belongs_to :board

  has_one :combat

  has_many :movement_preparation_steps, foreign_key: :actor_id
  has_many :corruptions, foreign_key: :actor_id, dependent: :destroy

  attr_accessor :final_attack, :final_defense

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
    @game_data_hero ||= @game_data_heroes.get(name_code)

    @game_data_hero.cards[card_number].pic_path
  end

  def draw_cards(board, nb_cards_to_draw, before_combat= false)
    transaction do
      cards = life_pool.shift(nb_cards_to_draw)
      self.hand += cards
      log_draw_cards!(board, cards.count, before_combat)
      save!
    end
  end

  #
  # Corruption methods
  #
  def discardeable_corruption_cards
    corruptions.where( 'favor_cost <= ?', favor )
  end

  def can_discard_corruption_cards?
    !corruption_card_discarded_this_turn && discardeable_corruption_cards.exists?
  end

  #
  # Heroes powers
  #
  def argalad_surrounding_monsters
    near_locations = GameData::LocationsPaths.new.get_connected_locations(location)
    board.monsters.where(location: near_locations)
  end

  private

  def log_draw_cards!(board, cards_drawn, before_combat= nil)
    action = ''
    action << 'combat.' if before_combat
    action << 'draw_cards'
    board.logs.create!(action: action, params: {name: name_code.to_sym, cards_drawn: cards_drawn, lp_cards: life_pool.count})
  end

end
