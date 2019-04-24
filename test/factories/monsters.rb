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

      name { 'Southron' }
      code { 'southron' }

      fortitude { 6 }
      strength { 3 }
      life { 4 }
    end

    factory :snaga do
      hand { GameData::MobsCards.new.get_deck( 'zealot' ) }
      attack_deck { :zealot }

      name { 'Snaga' }
      code { 'snaga' }

      fortitude { 6 }
      strength { 3 }
      life { 3 }
    end

    factory :huorn do
      hand { GameData::MobsCards.new.get_deck( 'behemoth' ) }
      attack_deck { :behemoth }

      name { 'Huorn' }
      code { 'huorn' }

      fortitude { 6 }
      strength { 7 }
      life { 7 }
    end

    factory :cave_troll do
      hand { GameData::MobsCards.new.get_deck( 'behemoth' ) }
      attack_deck { :behemoth }

      name { 'Cave troll' }
      code { 'cave_troll' }

      fortitude { 8 }
      strength { 8 }
      life { 8 }
    end

  end
end
