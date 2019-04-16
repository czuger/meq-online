require 'yaml'
require 'ostruct'

module GameData
  class MobsCards < Base

    def get_deck( attack_deck )
      check_attack_deck(attack_deck)
      @data[attack_deck][:deck].shuffle
    end

    def get_card_data( attack_deck, card_number )
      check_attack_deck(attack_deck)
      OpenStruct.new( @data[attack_deck][:data][card_number.to_i])
    end

    def get_card_pic_path( attack_deck, card_number )
      check_attack_deck(attack_deck)
      @data[attack_deck][:data][card_number.to_i][:pic_path]
    end

    private

    def check_attack_deck( attack_deck )
      raise "#{attack_deck} does not existe" unless @data[attack_deck]
    end

  end
end
