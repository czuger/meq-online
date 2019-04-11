FactoryBot.define do
  factory :board_plot do

    plot_position { 1 }
    plot_card { 1 }
    affected_location { 'dummy' }
    story_type { 'corruption' }
    story_advance { 3 }

  end
end
