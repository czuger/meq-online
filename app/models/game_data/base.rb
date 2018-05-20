require 'yaml'
require 'ostruct'

module GameData
  class Base

    def initialize
      p data_file = "#{Rails.root}/app/models/game_data/#{self.class::FILENAME}.yaml"
      @data = YAML.load_file(data_file)
    end

    def select_tag_data
      @data.map{ |k, v| [ v[:name], k ] }
    end

    def get( code )
      OpenStruct.new( @data[code] )
    end

  end
end
