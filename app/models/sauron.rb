class Sauron < Actor
  belongs_to :board
  belongs_to :user

  # validates_presence_of :plot_cards

  def name
    'Sauron'
  end

end

