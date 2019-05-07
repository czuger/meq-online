module GameEngine
  module HeroPeril

    def suffer_peril(board)
      # current_location_perilous? also instantiate @locations
      if current_location_perilous?(board)
        unless Hazard.lucky?(5)
          connected_locations = GameData::LocationsPaths.new.get_connected_locations(location)
          surrounding_locations_with_influence = board.influence.select { |k, v| connected_locations.include?(k) && v >= 0 }.count
          if wisdom <= surrounding_locations_with_influence
            favor_loss
            board.log(self, 'peril.shadow_background_memories', location_name: @locations.get(location).name)
          else
            board.log(self, 'peril.encounter_orcs_couts', location_name: @locations.get(location).name)
          end
          gain_random_corruption(board)
        end
      end
    end

    def favor_loss( amount = 1 )
      self.favor = [ favor - amount, 0 ].max
    end

    def gain_random_corruption(board)
      @game_data_corruption_cards = GameData::CorruptionCards.new

      corruption_card = draw_corruption_card(board)

      if corruption_card.immediate
        send(corruption_card.immediate, board)
      else
        send("get_corruption_#{corruption_card.modification}") if corruption_card.modification
        @game_data_corruption_cards.create_from_code!(board, self, corruption_card.code)
      end
    end

    def loose_corruption(board, corruption)
      if corruption.modification
        send("loose_corruption_#{corruption_card.modification}")
      elsif corruption.flaw
        self.flaws.delete(corruption.flaw)
      end

      board.corruption_discard << corruption.card_code
      corruption.destroy!
    end

    private

    def current_location_perilous?(board)
      @locations = GameData::Locations.new
      @locations.perilous?(location) || (board.influence[location].to_i > wisdom)
    end

  end
end