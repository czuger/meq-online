FactoryBot.define do
  factory :movement_preparation_step do
    board { nil }
    hero { nil }
    origine { 'MyString' }
    destination { 'MyString' }
    card_used { 1 }
    order { 1 }
    validation_required { false }
  end
end
