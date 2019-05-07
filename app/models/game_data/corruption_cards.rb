module GameData
  class CorruptionCards < Base

    def get(card_code)
      OpenStruct.new(@data[:cards][card_code].merge(code: card_code))
    end

    def deck
      @data[:deck]
    end

    def create_from_code!( board, card_code )
      card_data = @data[:cards][card_code]
      self.corruptions.create!( board: board, card_code: card_code, name: card_data[:name],
        favor_cost: card_data[:favor_cost], flaw: card_data[:flaw], modification: card_data[:modification] )
    end

  end
end