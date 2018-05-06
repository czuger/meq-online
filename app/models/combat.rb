class Combat < ApplicationRecord
  include AASM

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

  def log_increase_strength!( board, combat, hero )
    board.logs.create!( action: 'combat.inc_strength', params: {
        name: hero.name_code.to_sym,
        str: hero.strength, nstr: combat.temporary_strength } )
  end

end
