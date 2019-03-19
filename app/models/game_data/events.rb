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
      @data[:I].shuffle
    end

    def set_random_card(board)
      board.last_event_card = board.event_deck.shift
    end

  end
end