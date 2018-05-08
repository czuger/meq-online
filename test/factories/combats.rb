FactoryBot.define do
  factory :combat do
    board nil
    hero nil
    temporary_strength 1
    hero_cards_played "MyString"
    sauron_cards_played "MyString"
    monster :crebain
    sauron_hand { [] }
  end
end
