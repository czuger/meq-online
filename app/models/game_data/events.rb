module GameData
  class Events < Base

    def get_starting_event_deck
      @data[:I]
    end

    def set_random_card
      @data[:I].sample
    end

  end
end