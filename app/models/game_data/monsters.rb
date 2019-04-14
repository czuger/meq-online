require 'yaml'
require 'ostruct'

module GameData
  class Monsters < Base

    def get( name_code )
      monster = @data[:monsters][name_code.to_sym]
      cards = @data[:cards][monster[:attack_deck]]

      starting_deck = cards[:deck]
      cards_details = cards[:data].transform_values{ |v| OpenStruct.new( v ) }

      m = OpenStruct.new( monster )
      m.starting_deck = starting_deck.shuffle
      m.cards = cards_details

      m
    end

    def select_tag_data
      @data[:monsters].map{ |k, v| [ v[:name], k ] }.sort
    end

  end
end
