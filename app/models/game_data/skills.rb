require 'yaml'
require 'ostruct'

module GameData
  class Skills

    def initialize
      @data = YAML.load_file("#{Rails.root}/app/models/game_data/skills.yaml")
    end

    def starting_deck
      @data.keys
    end

    def get( card_code )
      OpenStruct.new( @data[card_code] )
    end

  end
end
