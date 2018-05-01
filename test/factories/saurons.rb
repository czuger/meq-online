FactoryBot.define do
  factory :sauron do
    board { create :board }
    user nil
    plot_cards { [] }
    shadow_cards { [] }
  end
end
