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

    def place_characters_and_influence(board, actor, card)
      characters = GameData::Characters.new
      char_data = @data[:I][card][:character]
      favors_data = @data[:I][card][:favors]

      if char_data
        characters.place_on_map(board, actor, char_data.name, char_data.location)
      end

      board.favors += favors_data
    end

  end
end