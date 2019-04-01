FactoryBot.define do
  factory :board_plot do
    plot_position { 1 }
    plot_card { 1 }
    affected_location { "MyString" }
    story_type { "MyString" }
    story_increase { "" }
  end
end
