require 'yaml'
require 'ostruct'

module GameData
  class LocationsMonsters < Base

    LOCATION_CONVERSION = { orange: :orange, yellow: :orange, purple: :purple, light_blue: :dark_blue,
                            dark_blue: :dark_blue, white: :purple, brown: :brown, red: :brown, light_green: :dark_green,
                            dark_green: :dark_green }

    def fill_board(board)
      @data.each do |key, values|
        board.update( monster_pool_key(key) => values )
      end
    end

    def place_new_monster(board, location)
      game_data_locations = GameData::Locations.new
      location_color = game_data_locations.get(location).color_code

      monster_location_color = LOCATION_CONVERSION[location_color]
      monster_color_key = monster_pool_key(monster_location_color)

      current_monsters_list = board.send(monster_color_key)
      monster = current_monsters_list.sample
      current_monsters_list.slice!( current_monsters_list.index( monster ) )
      board.update( monster_color_key => current_monsters_list )

      board.create_monster( monster, location, monster_color_key )
    end

    def place_monster_back_to_monster_pool(board, monster)
      current_monsters_list = board.send(monster.pool_key)
      current_monsters_list << monster.code
      board.update( monster.pool_key => current_monsters_list )
      monster.destroy
    end

    private

    def monster_pool_key(color)
      "monsters_pool_#{color}"
    end

  end
end
