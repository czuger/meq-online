require 'yaml'
require 'ostruct'

module GameData
  class Monsters

    def initialize
      @data = YAML.load_file("#{Rails.root}/app/models/game_data/monsters.yaml")
    end

    def get( name_code )
      monster = @data[:monsters][name_code]
      cards = @data[:cards][monster[:attack_deck]]

      starting_deck = cards[:deck]
      cards_details = cards[:data].transform_values{ |v| OpenStruct.new( v ) }

      m = OpenStruct.new( monster )
      m.starting_deck = starting_deck
      m.cards = cards_details

      m
    end

  end
end
