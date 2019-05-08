FactoryBot.define do

  factory :corruption do

    factory :isolated do
      name { 'Isolated' }
      card_code { 11 }
      favor_cost { 2 }
      flaw { 'isolated' }
      modification { nil }
    end

    factory :dispairing do
      name { 'Dispairing' }
      card_code { 9 }
      favor_cost { 2 }
      flaw { 'dispairing' }
      modification { nil }
    end

  end

end
