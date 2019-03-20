FactoryBot.define do
  factory :sauron do
    name { 'Sauron' }
    plot_cards { [ 0, 1, 2 ] }
    shadow_cards { [ 0, 1, 2 ] }
    drawn_plot_cards { [ 0, 1, 2 ] }
    drawn_shadow_cards { [ 0, 1, 2 ] }
    active { true }
  end
end
