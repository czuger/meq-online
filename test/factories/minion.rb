FactoryBot.define do
  factory :minion do
    code { 'black_serpent' }
    location { 'near_harad' }

    fortitude { 8 }
    strength { 8 }
    life { 8 }
    max_life { 8 }

    name { 'The black serpent' }

    attack_deck { :ravager }

    factory :mouth_of_sauron do
      code { 'mouth_of_sauron' }
      location { 'near_harad' }

      fortitude { 6 }
      strength { 6 }
      life { 9 }
      max_life { 9 }

      name { 'Mouth of Sauron' }

      attack_deck { :zealot }
    end

    factory :black_serpent do
      code { 'black_serpent' }
      location { 'near_harad' }

      fortitude { 9 }
      strength { 8 }
      life { 11 }
      max_life { 11 }

      name { 'Black serpent' }

      attack_deck { :ravager }
    end
  end
end