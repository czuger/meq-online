require 'yaml'
require 'ostruct'

module GameData
  class LocationsPaths < Base

    attr_reader :data

    def initialize
      @data = YAML.load_file("#{Rails.root}/app/models/game_data/locations_paths.yaml")
    end

    # Return the codes of surrounding locations
    #
    # @param name_code [String] the code of the current location.
    # @return [Array[String]] all codes of surrounding locations.
    def get_connected_locations( name_code )
      @data[name_code][:destinations].map{ |e| e[:dest] }
    end

    def get_connected_locations_for_select( name_code, has_horse )
      @path_difficulty_decrease = has_horse ? 1 : 0
      @data[name_code][:destinations].map{ |e| [ "#{e[:dest].to_s.humanize} - #{e[:path_type]}(#{get_path_difficulty(e)})", e[:dest] ] }
    end

    def path_data( source, required_dest )
      @data[source][:destinations].each do |destination|
        if destination[:dest] == required_dest
          return [ destination[:path_type], destination[:difficulty] ]
        end
      end
      raise "Path from #{source} to #{required_dest} does not exist."
    end

    def exist?( name_code )
      @data.has_key?( name_code )
    end

    def delete!( location_name )
      @data.delete( location_name )
    end

    private

    def get_path_difficulty(e)
      [e[:difficulty]-@path_difficulty_decrease, 1].max
    end

  end
end
