class Hero < ApplicationRecord

  belongs_to :user
  belongs_to :board

  has_one :combat

  serialize :life_pool
  serialize :rest_pool
  serialize :damage_pool
  serialize :hand

  def log_movement!( board, card )
    board.logs.create!( action: :move, params: {
        name: name_code.to_sym,
        from: location.to_sym,
        to: location.to_sym,
        card: card } )
  end

  def log_draw_cards!( board, cards_drawn )
    board.logs.create!( action: :draw_cards, params: {
        name: name_code.to_sym,
        cards_drawn: cards_drawn, lp_cards: life_pool.count } )
  end

end
