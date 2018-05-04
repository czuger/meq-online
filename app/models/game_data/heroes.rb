require 'yaml'
require 'ostruct'

module GameData
  class Heroes

    def initialize
      @data = YAML.load_file("#{Rails.root}/app/models/game_data/heroes.yaml")
    end

    def get( name_code )
      hero = OpenStruct.new( @data[name_code.to_sym] )
      hero.cards.transform_values!{ |v| OpenStruct.new( v ) }
      hero
    end

    def delete_heroes!( heroes_codes_names_list )
      @data.reject!{ |k, _| heroes_codes_names_list.include?( k ) }
    end

    def select_tag_data
      @data.map{ |k, v| [ v[:name], k ] }
    end

  end
end
