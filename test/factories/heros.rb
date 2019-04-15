FactoryBot.define do
  factory :hero do

    name_code {'argalad'}
    name {'Argalad'}
    fortitude {1}
    strength {5}
    agility {3}
    wisdom {1}
    location {'the_shire'}
    hand { [ 1 ] }
    life_pool { [ 2, 3, 4, 5, 6, 7, 8, 9, 10 ] }
    rest_pool { [] }
    damage_pool { [] }
    current_quest { 1 }

    active { true }
    playing_order { 0 }
  end
end
