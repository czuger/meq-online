class Hero < ApplicationRecord

  belongs_to :user
  belongs_to :board

  has_one :combat

  serialize :life_pool
  serialize :rest_pool
  serialize :damage_pool
  serialize :hand

  def draw_cards( board, nb_cards_to_draw, before_combat= false )
    transaction do
      cards = life_pool.shift(nb_cards_to_draw)
      self.hand += cards
      log_draw_cards!( board, cards.count, before_combat)
      save!
    end
  end

  #
  # Log methods
  #
  def log_movement!( board, card )
    board.logs.create!( action: :move, params: { name: name_code.to_sym, from: location.to_sym, to: location.to_sym,
        card: card } )
  end

  def log_draw_cards!( board, cards_drawn, before_combat= nil )
    action = ''
    action << 'combat.' if before_combat
    action << 'draw_cards'
    board.logs.create!( action: action, params: { name: name_code.to_sym, cards_drawn: cards_drawn, lp_cards: life_pool.count } )
  end

end
