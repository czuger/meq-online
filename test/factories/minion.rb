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
    hand { [2, 2, 6, 7, 0, 0, 0, 10, 6, 4, 4, 2, 6, 10, 4, 5, 5, 7, 5, 7, 0, 5, 2, 4, 10] }

    factory :mouth_of_sauron do
      code { 'mouth_of_sauron' }
      location { 'near_harad' }

      fortitude { 6 }
      strength { 6 }
      life { 9 }
      max_life { 9 }

      name { 'Mouth of Sauron' }

      attack_deck { :zealot }
      hand { [3, 6, 10, 3, 7, 9, 6, 6, 8, 10, 8, 1, 3, 4, 4, 9, 9, 4, 1, 1, 1, 8, 7, 10, 7] }
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
      hand { [2, 2, 6, 7, 0, 0, 0, 10, 6, 4, 4, 2, 6, 10, 4, 5, 5, 7, 5, 7, 0, 5, 2, 4, 10] }
    end

  end
end