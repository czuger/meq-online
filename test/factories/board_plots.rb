FactoryBot.define do
  factory :board_plot do

    plot_position { 1 }
    plot_card { 1 }
    affected_location { 'the_shire' }
    story_type { 'corruption' }
    story_advance { 3 }
    favor_to_discard { 3 }

    factory :conquest_plot do
      plot_card { 1 }
      affected_location { 'the_shire' }
      story_type { 'conquest' }
      story_advance { 2 }
      favor_to_discard { 3 }
    end

  end
end
