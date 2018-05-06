class Combat < ApplicationRecord
  include AASM

  aasm do
  end
  belongs_to :board
  belongs_to :hero

  serialize :hero_cards_played
  serialize :sauron_cards_played

  aasm do
    state :hero_choices, :initial => true
    state :started

    event :start do
      transitions :from => :hero_choices, :to => :started
    end
  end

end
