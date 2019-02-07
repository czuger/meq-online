require 'yaml'
require 'ostruct'

module GameData
  class LocationsPaths < Base

    attr_reader :data

    def initialize
      @data = YAML.load_file("#{Rails.root}/app/models/game_data/locations_paths.yaml")
    end

    def get_connected_locations( name_code )
      @data[name_code][:destinations].map{ |e| e[:dest] }
    end

    def get_connected_locations_for_select( name_code )
      @data[name_code][:destinations].map{ |e| [ [ e[:dest].to_s.humanize, e[:path_type], e[:difficulty] ].join( ', '), e[:dest] ] }
    end

    def exist?( name_code )
      @data.has_key?( name_code )
    end

    def delete!( location_name )
      @data.delete( location_name )
    end

  end
end
