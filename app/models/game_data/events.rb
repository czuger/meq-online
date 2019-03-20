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
      card = board.event_deck.shift

      # raise "On board #{board.id} : Event deck is empty" unless card

      board.last_event_card = card

      board.log( nil, 'event.choose_card', { event_card: "I/#{card}.png" } )
    end

    def place_characters_and_influence(board, card)
      characters = GameData::Characters.new
      char_data = @data[:I][card][:character]
      favors_data = @data[:I][card][:favors]

      if char_data
        characters.place_on_map(board, nil, char_data.name, char_data.location)
      end

      board.favors += favors_data

      favors_data.each do |fd|
        board.log( nil, 'favor.place', location: fd.to_s.humanize )
      end
    end

  end
end