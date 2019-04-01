FactoryBot.define do
  factory :board_plot do

    plot_position { 1 }
    plot_card { 1 }
    affected_location { 'dummy' }
    story_type { 'dummy' }
    story_advance { 1 }

  end
end
