require 'yaml'
require 'ostruct'

module GameData
  class Heroes < Base

    def initialize
      @data = YAML.load_file("#{Rails.root}/app/models/game_data/heroes.yaml")
    end

    def get( name_code )
      raise "#{name_code} not found in #{@data.keys}" unless @data.keys.include?(name_code.to_sym)
      hero = OpenStruct.new( @data[name_code.to_sym] )
      hero.cards.transform_values!{ |v| OpenStruct.new( v ) }
      hero
    end

    def delete_heroes!( heroes_codes_names_list )
      @data.reject!{ |k, _| heroes_codes_names_list.include?( k ) }
    end

    def select_heroes_from_board!( board )
      heroes_codes_names_list = board.heroes.map{ |h| h.name_code.to_sym }
      @data.select!{ |k, _| heroes_codes_names_list.include?( k ) }
    end

  end

end
