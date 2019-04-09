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

    def suffer_peril!(board)
      transaction do
        if current_location_perilous?
          case Hazard.d4
            when 1
              board.log( self, 'peril.pass_trough' )
            when 2
              hand_to_life(hand.sample)
              board.log( self, 'peril.lose_card' )
            when 3
              self.favor -= 1
              board.log( self, 'peril.lose_favor' )
            when 4
              hand_to_life(hand.sample)
              self.favor -= 1
              board.log( self, 'peril.lose_favor_and_card' )
            else
              raise 'Hazard is not working, arghhhhh !!!'
          end
        end
      end
    end

    def hand_to_rest(cards)
      cards_from_hand(:rest, cards)
    end

    def hand_to_life(cards)
      cards_from_hand(:life, cards)
    end

    private

    def current_location_perilous?(board)
      locations = GameData::Locations.new
      locations.perilous?(location) || board.influence[location] > wisdom
    end

    def cards_from_hand(pool, cards)
      cards = [ cards ] if cards.is_a? Integer
      self.hand -= cards
      self.send(pool) += cards
    end

  end

end
