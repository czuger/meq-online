class Combat < ApplicationRecord
  include AASM

  belongs_to :board
  belongs_to :hero, class_name: 'Actor', foreign_key: :actor_id
  belongs_to :mob

  # serialize :hero_cards_played
  # serialize :sauron_cards_played
  # serialize :sauron_hand

  # aasm do
  #   state :hero_choices, :initial => true
  #   state :started
  #
  #   event :start do
  #     transitions :from => :hero_choices, :to => :started
  #   end
  # end

end
