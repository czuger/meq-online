require 'yaml'
require 'ostruct'

module GameData
  class Heroes < Base

    def get( name_code )
      raise "#{name_code} not found in #{@data.keys}" unless @data.keys.include?(name_code.to_sym)
      hero = OpenStruct.new( @data[name_code.to_sym] )
      hero.cards.transform_values!{ |v| OpenStruct.new( v ) }
      hero
    end

    def get_card_number_by_name( hero_name_code, card_name )
      raise "#{hero_name_code} not found in #{@data.keys}" unless @data.keys.include?(hero_name_code.to_sym)
      @data[hero_name_code][:cards].each do |k, v|
        return k if v[:name] == card_name.to_sym
      end
      raise "Card #{card_name} not found."
    end

    def get_card_data( name_code, card_number )
      return nil unless card_number
      OpenStruct.new( @data[name_code.to_sym][:cards][card_number.to_i] )
    end

    def get_starting_hand( name_code )
      return nil unless card_number
      @data[name_code.to_sym][:cards]
    end

    def get_deck( name_code )
      @data[name_code][:starting_deck].shuffle
    end

    def get_deck_with_card_name( name_code )
      @data[name_code][:starting_deck].uniq.map{ |c| Hash[ get_card_data( name_code, c ).name.to_s, c ] }.sort_by{ |e| e.first }
    end

    def delete_heroes!( heroes_codes_names_list )
      @data.reject!{ |k, _| heroes_codes_names_list.include?( k ) }
    end

    def heroes
      @data.keys
    end

    def select_heroes_from_board!( board )
      heroes_codes_names_list = board.heroes.map{ |h| h.name_code.to_sym }
      @data.select!{ |k, _| heroes_codes_names_list.include?( k ) }
    end

    def common_cards_names
      @data.values.map{ |h| h[:cards].values.map{ |c| c[:name].to_s } }.flatten
    end
  end
end
