FactoryBot.define do
  factory :monster do
    pool_key { 'monsters_pool_orange' }
    code { 'agent' }
    name { 'Agent' }
    location { 'the_nindale' }

    fortitude { 5 }
    strength { 3 }
    life { 3 }

    hand { GameData::MobsCards.new.get_deck( 'zealot' ) }
    attack_deck { :zealot }

    factory :orc do
      name { 'Orc' }
      code { 'orc' }

      fortitude { 5 }
      strength { 4 }
      life { 3 }
    end

    factory :southron do
      hand { GameData::MobsCards.new.get_deck( 'ravager' ) }
      attack_deck { :ravager }

      fortitude { 6 }
      strength { 3 }
      life { 4 }
    end

  end
end
