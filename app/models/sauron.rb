class Sauron < Player
  belongs_to :board
  belongs_to :user
  serialize :plot_cards
  serialize :shadow_cards

  def name
    'Sauron'
  end

end
