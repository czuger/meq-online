require 'yaml'
require 'ostruct'

module GameData
  class MobsCards < Base

    KEPT_CARDS = ["Aimed Shot", "Attack of Opportunity", "Charge", "Evade", "Fall Back", "Hack", "Parry", "Ranged Strike", "Reckless", "Smash"]

    def get_deck( attack_deck )
      check_attack_deck(attack_deck)
      @data[attack_deck][:deck].shuffle
    end

    def get_card_data( attack_deck, card_number )
      return nil unless card_number
      check_attack_deck(attack_deck)
      OpenStruct.new( @data[attack_deck][:data_and_rules][card_number.to_i])
    end

    def get_card_pic_path( attack_deck, card_number )
      check_attack_deck(attack_deck)
      @data[attack_deck][:data_and_rules][card_number.to_i][:pic_path]
    end

    def common_cards_names
      @data.values.map{ |h| h[:'data and rules'].values.map{ |c| c[:name].to_s } }.flatten
    end

    def kept_cards_per_mob
      mob_hash = Hash[@data.keys.map{ |k| [ k, [] ] }]
      mob_hash.keys.each do |hk|
        @data[hk][:data_and_rules].each do |ck, cv|
          mob_hash[hk] << ck if KEPT_CARDS.include?( cv[:name].to_s )
        end
        cards_backup = mob_hash[hk]
        cards = cards_backup.clone
        cards_pool = []
        while cards_pool.count < 25
          cards_pool << cards.shift
          cards = cards_backup.clone if cards.empty?
        end
        mob_hash[hk] = cards_pool
      end
      File.open( 'mob_hash.yaml', 'w' ){ |f| f.puts mob_hash.to_yaml }
    end

    private

    def check_attack_deck( attack_deck )
      raise "#{attack_deck} does not existe" unless @data[attack_deck]
    end

  end
end
