module GameData
  class Characters < Base

    FILENAME = 'characters'

    def list
      @data.map{ |k, v| OpenStruct.new( name: v[:name], name_code: k ) }
    end

    def name( char )
      @data[char][:name]
    end

    def exist?( code )
      @data.has_key? code
    end

  end
end