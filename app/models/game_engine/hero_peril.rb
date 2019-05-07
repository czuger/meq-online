module GameEngine
  module HeroPeril

    def suffer_peril!(board)
      # transaction do
      #   # current_location_perilous? also instantiate @locations
      #   if current_location_perilous?(board)
      #     unless Hazard.lucky?(5)
      #       connected_locations = GameData::LocationsPaths.new.get_connected_locations(location)
      #       surrounding_locations_with_influence = board.influence.select { |k, v| connected_locations.include?(k) && v >= 0 }.count
      #       if wisdom <= surrounding_locations_with_influence
      #         favor loss
      #       end
      #
      #       coruuption
      #
      #     else
      #       board.log(self, 'peril.pass_trough', location_name: @locations.get(location).name)
      #     end
      #     case
      #       when 1
      #
      #       when 2
      #         hand_to_damages(hand.sample)
      #         board.log(self, 'peril.lose_card', location_name: @locations.get(location).name)
      #       when 3
      #         self.favor -= 1
      #         board.log(self, 'peril.lose_favor', location_name: @locations.get(location).name)
      #       when 4
      #         hand_to_damages(hand.sample)
      #         self.favor -= 1
      #         board.log(self, 'peril.lose_favor_and_card', location_name: @locations.get(location).name)
      #       else
      #         raise 'Hazard is not working, arghhhhh !!!'
      #     end
      #   end
      #   self.save!
      # end
    end

    private

    def current_location_perilous?(board)
      @locations = GameData::Locations.new
      @locations.perilous?(location) || (board.influence[location] && board.influence[location] > wisdom)
    end

  end
end