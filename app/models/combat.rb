class Combat < ApplicationRecord
  belongs_to :board
  belongs_to :hero

  serialize :hero_cards_played
  serialize :sauron_cards_played
end
