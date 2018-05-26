class Sauron < Actor
  belongs_to :board
  belongs_to :user

  serialize :plot_cards
  serialize :shadow_cards
  serialize :drawn_plot_cards
  serialize :drawn_shadow_cards

  # validates_presence_of :plot_cards

  def name
    'Sauron'
  end

end

