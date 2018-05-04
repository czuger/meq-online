require 'yaml'
require 'ostruct'

module GameData
  class Locations

    def initialize
      @data = YAML.load_file("#{Rails.root}/app/models/game_data/locations.yaml")
    end

    def get( name_code )
      OpenStruct.new( @data[name_code] )
    end

    def delete!( location_name )
      @data.delete( location_name.to_sym )
    end

    def select_tag_data
      @data.map{ |k, v| [ v[:name], k ] }
    end

  end
end
