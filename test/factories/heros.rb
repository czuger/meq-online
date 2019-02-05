FactoryBot.define do
  factory :hero do

    name_code 'argalad'
    name 'Argalad'
    fortitude 1
    strength 1
    agility 1
    wisdom 1
    location 'harlingdon'
    hand { [ 1 ] }
    life_pool { [] }
    rest_pool { [] }
    damage_pool { [] }

  end
end
