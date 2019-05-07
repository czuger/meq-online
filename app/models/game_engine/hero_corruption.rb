module GameEngine
  module HeroCorruption

    def distraught?
      flaw?( __method__ )
    end

    def cowardly?
      flaw?( __method__ )
    end

    def dispairing?
      flaw?( __method__ )
    end

    def isolated?
      flaw?( __method__ )
    end

    private

    def get_corruption_helpless
      self.agility -= 1
    end

    def get_corruption_derangd
      self.wisdom -= 1
    end

    def get_corruption_weary
      self.fortitude -= 1
    end

    def get_corruption_weak
      self.strength -= 1
    end

    def get_corruption_helpless
      self.agility -= 1
    end

    def get_corruption_dispairing
      self.favor = [favor, 3].min
    end

    def loose_corruption_derangd
      self.wisdom += 1
    end

    def loose_corruption_weary
      self.fortitude += 1
    end

    def loose_corruption_weak
      self.strength += 1
    end

    def loose_corruption_helpless
      self.agility += 1
    end

    def loose_corruption_dispairing
    end

    def careless(board)
      favor_loss(2)
      board.corruption_deck << 10
      shuffle_corruption_discard_in_deck(board)
    end

    def draw_corruption_card(board)
      shuffle_corruption_discard_in_deck(board) if board.corruption_deck.empty?
      @game_data_corruption_cards.get(board.corruption_deck.shift)
    end

    def shuffle_corruption_discard_in_deck(board)
      board.corruption_deck += board.corruption_discard
      board.corruption_deck.shuffle!
      board.corruption_discard = []
    end

    def flaw?( method_name )
      flaw = method_name.to_s[0..-2]
      self.corruptions.where( flaw: flaw ).exists?
    end

  end
end