module GameData
  class Events < Base

    attr_reader :data

    def initialize
      super
      @data.values.each do |level|
        level.values.each do |content|
          content[:character] = OpenStruct.new( content[:character] ) if content[:character]
        end
      end
    end

    def get_starting_event_deck
      @data[:I].keys.shuffle
    end

    def set_random_card(board)
      board.last_event_card = board.event_deck.shift
    end

    def place_characters_and_influence(board)
      if @data[:I][:character]
      end

    end

  end
end