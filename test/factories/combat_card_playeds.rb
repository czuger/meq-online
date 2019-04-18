FactoryBot.define do
  factory :combat_card_played do
    combat { nil }
    type { "" }
    card { 1 }
    pic_path { "MyString" }
    name { "MyString" }
    power { "MyString" }
    strength_cost { 1 }
    printed_attack { 1 }
    final_attack { 1 }
    printed_defense { 1 }
    final_defense { 1 }
    cancelled { false }
  end
end
