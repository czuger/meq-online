require 'yaml'
require 'ostruct'

module GameData
  class Heroes

    def initialize
      @data = YAML.load_file("#{Rails.root}/app/models/game_data/heroes.yaml")
    end

    def get( name_code )
      hero = OpenStruct.new( @data[name_code] )
      hero.cards.transform_values{ |v| OpenStruct.new( v ) }
      hero
    end

  end
end
