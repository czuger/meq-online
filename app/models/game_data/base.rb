require 'yaml'
require 'ostruct'

module GameData
  class Base

    def initialize
      data_file_name = self.class::FILENAME if defined? self.class::FILENAME
      data_file_name ||= self.class.name.demodulize.underscore

      data_file = "#{Rails.root}/app/models/game_data/#{data_file_name}.yaml"
      # p data_file
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
