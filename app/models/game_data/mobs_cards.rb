require 'yaml'
require 'ostruct'

module GameData
  class MobsCards < Base

    def get_deck( attack_deck )
      check_attack_deck(attack_deck)
      @data[attack_deck][:deck].shuffle
    end

    def get_card_number_by_name( attack_deck, card_name )
      check_attack_deck(attack_deck)
      @data[attack_deck][:cards].each do |k, v|
        return k if v[:name] == card_name
      end
      raise "Card #{card_name} not found."
    end

    def get_card_data( attack_deck, card_number )
      return nil unless card_number
      check_attack_deck(attack_deck)
      OpenStruct.new( @data[attack_deck][:cards][card_number.to_i])
    end

    def get_deck_with_card_name( name_code )
      @data[name_code][:deck].uniq.map{ |c| Hash[ get_card_data( name_code, c ).name.to_s, c ] }.sort_by{ |e| e.first }
    end

    def get_card_pic_path( attack_deck, card_number )
      check_attack_deck(attack_deck)
      @data[attack_deck][:cards][card_number.to_i][:pic_path]
    end

    def common_cards_names
      @data.values.map{ |h| h[:cards].values.map{ |c| c[:name].to_s } }.flatten
    end

    private

    def check_attack_deck( attack_deck )
      raise "#{attack_deck} does not existe" unless @data[attack_deck]
    end

  end
end
