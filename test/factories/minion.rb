FactoryBot.define do
  factory :minion do
    code { 'black_serpent' }
    location { 'near_harad' }

    fortitude { 8 }
    strength { 8 }
    life { 8 }

    name { 'The black serpent' }

    attack_deck { :ravager }
  end
end
