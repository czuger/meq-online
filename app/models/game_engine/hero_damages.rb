module GameEngine
  module HeroDamages

    def deal_damages(damages_amount)
      damages_amount = [damages_amount, 0].max

      self.temporary_damages += damages_amount
      self.damages_taken_this_turn += damages_amount
    end

    def rest!
      self.life_pool += rest_pool
      self.rest_pool = []
      self.life_pool.shuffle
      save!
    end

    def heal!
      transaction do
        rest!

        self.life_pool += damage_pool
        self.damage_pool = []
        self.life_pool.shuffle
        save!
      end
    end

    def hand_to_rest(cards)
      discard_cards(cards) {|c| self.rest_pool += c}
    end

    def hand_to_damages(cards)
      discard_cards(cards) {|c| self.damage_pool += c}
    end

    private

    def current_location_perilous?(board)
      @locations = GameData::Locations.new
      @locations.perilous?(location) || (board.influence[location] && board.influence[location] > wisdom)
    end

    # Used only with hand_to_rest and hand_to_life
    def discard_cards(cards)
      cards = [] unless cards
      cards = [cards] if cards.is_a? Integer

      # We must remove cards one by one, otherwise we would remove too much cards
      # remember that [2, 3, 3, 4] - [2, 3] = [4] and not [3, 4]
      cards.each do |card|
        self.hand.slice!(self.hand.index(card))
      end

      yield(cards)
    end

  end
end