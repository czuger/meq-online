module GameData
  class Events < Base

    def get_starting_event_deck
      @data[:I].shuffle
    end

    def set_random_card(board)
      board.last_event_card = @board.event_deck.shift
    end

  end
end