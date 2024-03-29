FactoryBot.define do
  factory :combat_card_played do

    factory :argalad_quick_shot do
      type { 'CombatCardPlayedHero' }
      card_type { 'ranged' }
      card { 0 }
      pic_path { 'argalad/quick_shot.jpg' }
      name { 'Quick Shot' }
      power { 'quick_shot' }
      strength_cost { 0 }
      printed_attack { 1 }
      printed_defense { 0 }
      cancelled { false }
    end

    factory :behemoth_precision do
      type { 'CombatCardPlayedMob' }
      card_type { 'ranged' }
      card { 0 }
      pic_path { 'behemoth/precision.jpg' }
      name { 'Precision' }
      power { 'precision' }
      strength_cost { 2 }
      printed_attack { 2 }
      printed_defense { 1 }
      cancelled { false }
    end

    factory :mob_exhausted do
      type { 'CombatCardPlayedMob' }
      card_type { 'exhausted' }
      card { -1 }
      pic_path { 'exhausted' }
      name { 'Exhausted' }
      power { 'none' }
      strength_cost { 0 }
      printed_attack { 0 }
      printed_defense { 0 }
      cancelled { true }

      factory :hero_exhausted do
        type { 'CombatCardPlayedHero' }
      end
    end

  end
end