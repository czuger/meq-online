module GameData
  class Characters < Base

    FILENAME = 'characters'

    def list
      @data.map{ |k, v| OpenStruct.new( name: v[:name], name_code: k ) }
    end

    def name( char )
      @data[char][:name]
    end

  end
end