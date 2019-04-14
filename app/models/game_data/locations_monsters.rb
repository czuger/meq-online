require 'yaml'
require 'ostruct'

module GameData
  class LocationsMonsters < Base

    LOCATION_CONVERSION = { orange: :orange, yellow: :orange, purple: :purple, light_blue: :dark_blue,
                            dark_blue: :dark_blue, white: :purple, brown: :brown, red: :brown, light_green: :dark_green,
                            dark_green: :dark_green }

    def fill_board!(board)
      @data.each do |key, values|
        board.update( "monsters_pool_#{key}" => values )
      end
    end

    def fill_board!(board)
      @data.each do |key, values|
        board.update( "monsters_pool_#{key}" => values )
      end
    end

  end
end
