FactoryBot.define do
  factory :monster do
    pool_key { 'monsters_pool_orange' }
    code { 'agent' }
    name { 'Agent' }
    location { 'the_nindale' }

    fortitude { 5 }
    strength { 3 }
    life { 3 }
    max_life { 3 }

    hand { GameData::MobsCards.new.get_deck( 'zealot' ) }
    attack_deck { :zealot }

    factory :orc do
      name { 'Orc' }
      code { 'orc' }

      fortitude { 5 }
      strength { 4 }
      life { 3 }
      max_life { 3 }
      hand { [6, 7, 8, 3, 10, 9, 10, 6, 3, 7, 8, 1, 1, 6, 1, 4, 1, 9, 9, 10, 8, 4, 7, 3, 4] }
    end

    factory :southron do
      hand { GameData::MobsCards.new.get_deck( 'ravager' ) }
      attack_deck { :ravager }

      name { 'Southron' }
      code { 'southron' }

      fortitude { 6 }
      strength { 3 }
      life { 4 }
      max_life { 4 }
    end

    factory :snaga do
      hand { GameData::MobsCards.new.get_deck( 'zealot' ) }
      attack_deck { :zealot }

      name { 'Snaga' }
      code { 'snaga' }

      fortitude { 6 }
      strength { 3 }
      life { 3 }
      max_life { 3 }
    end

    factory :huorn do
      hand { GameData::MobsCards.new.get_deck( 'behemoth' ) }
      attack_deck { :behemoth }

      name { 'Huorn' }
      code { 'huorn' }

      fortitude { 6 }
      strength { 7 }
      life { 7 }
      max_life { 7 }
    end

    factory :cave_troll do
      hand { GameData::MobsCards.new.get_deck( 'behemoth' ) }
      attack_deck { :behemoth }

      name { 'Cave troll' }
      code { 'cave_troll' }

      fortitude { 8 }
      strength { 8 }
      life { 8 }
      max_life { 8 }
    end

    factory :nothing do
      hand { [] }
      attack_deck { :none }

      name { 'Nothing' }
      code { 'nothing' }

      fortitude { 0 }
      strength { 0 }
      life { 0 }
      max_life { 0 }

      hand { [] }
    end

  end
end
