class Sauron < ApplicationRecord
  belongs_to :board
  belongs_to :user

  serialize :plot_cards
  serialize :shadow_cards
  serialize :drawn_plot_cards
  serialize :drawn_shadow_cards

end
