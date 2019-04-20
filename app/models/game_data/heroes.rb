require 'yaml'
require 'ostruct'

module GameData
  class Heroes < Base

    # given by :
    # ( GameData::Heroes.new.common_cards_names + GameData::MobsCards.new.common_cards_names ).inject(Hash.new(0)) { |total, e| total[e] += 1 ;total}.select{ |e, i| i >= 3 }.map{ |e| e.first }.sort
    KEPT_CARDS = [ 'Concentrate', 'Deadly Finesse', 'Overdraw', 'Precision', 'Aimed Shot', 'Attack of Opportunity', 'Charge', 'Evade', 'Fall Back', 'Hack', 'Parry', 'Ranged Strike', 'Reckless', 'Smash']

    def get( name_code )
      raise "#{name_code} not found in #{@data.keys}" unless @data.keys.include?(name_code.to_sym)
      hero = OpenStruct.new( @data[name_code.to_sym] )
      hero.cards.transform_values!{ |v| OpenStruct.new( v ) }
      hero
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

    def kept_cards_per_heroes
      heroes_hash = Hash[@data.keys.map{ |k| [ k, [] ] }]
      heroes_hash.keys.each do |hk|
        @data[hk][:cards].each do |ck, cv|
          heroes_hash[hk] << ck if KEPT_CARDS.include?( cv[:name].to_s )
        end
        cards_backup = heroes_hash[hk]
        cards = cards_backup.clone
        cards_pool = []
        while cards_pool.count < 25
          cards_pool << cards.shift
          cards = cards_backup.clone if cards.empty?
        end
        heroes_hash[hk] = cards_pool
      end
      File.open( 'heroes.yaml', 'w' ){ |f| f.puts heroes_hash.to_yaml }
    end

  end

end
