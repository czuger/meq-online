FactoryBot.define do
  factory :log do
    board { nil }
    action { 'draw_cards' }
    params { { name: :argalad, cards_drawn: 1, lp_cards: 1 } }
  end
end

