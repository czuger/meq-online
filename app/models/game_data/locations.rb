require 'yaml'
require 'ostruct'

module GameData
  class Locations < Base

    def initialize
      @data = YAML.load_file("#{Rails.root}/app/models/game_data/locations.yaml")
    end

    def get( name_code )
      OpenStruct.new( @data[name_code.to_sym] )
    end

    def exist?( name_code )
      @data.has_key?( name_code.to_sym )
    end

    def delete!( location_name )
      @data.delete( location_name.to_sym )
    end

    def list_by_region
      @data.map{ |k, v| OpenStruct.new( name_code: k, name: v[:name], region: v[:region] ) }.sort_by { |e| [e.region, e.name ] }
    end

    def position_list
      @data.map{ |k, v| OpenStruct.new( name_code: k, x: v[:pos_x], y: v[:pos_y] ) }
    end

    def alpha_select_tag_data
      @data.map{ |k, v| [ v[:name], k ] }.sort
    end

  end
end
