module GameData
  class Characters < Base

    # Characters uses locations. If locations is already defined in your code you can pass it as a parameter.
    # To avoid multiple allocation.
    def initialize(locations= nil)
      super()
      @locations= locations || GameData::Locations.new
    end

    def list
      @data.map{ |k, v| OpenStruct.new( name: v[:name], name_code: k ) }
    end

    def name( char )
      @data[char][:name]
    end

    def exist?( code )
      @data.has_key? code
    end

    def remove_from_map(board, actor, character_code)
      board.characters[character_code] = nil
      board.log( actor, 'character.remove', name: name( character_code ) )
    end

    def place_on_map(board, character_code, location_code)
      board.characters[character_code] = location_code
      board.log( actor, 'character.place', name: name( char ), location: @locations.get( character_code ).name )
    end

  end
end