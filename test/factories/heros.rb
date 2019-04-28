FactoryBot.define do
  factory :hero do

    name_code {'argalad'}
    name {'Argalad'}
    fortitude {5}
    strength {5}
    agility {3}
    wisdom {2}
    location {'the_shire'}
    hand { [] }
    life_pool { [] }
    rest_pool { [] }
    damage_pool { [] }
    current_quest { 1 }

    active { true }
    playing_order { 0 }

    factory :eometh do
      name_code {'eometh'}
      name {'Eometh'}
    end

    after(:create) do |hero|
      cards = GameData::Heroes.new.get_deck( :argalad )
      hero.hand = cards.shift( hero.fortitude )
      hero.life_pool = cards
      hero.save!
    end
  end
end