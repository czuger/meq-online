class Board < ApplicationRecord
  include AASM

  has_many :logs, dependent: :destroy

  has_many :heroes, dependent: :destroy
  has_one :sauron, dependent: :destroy

  has_and_belongs_to_many :users

  serialize :influence

  serialize :current_plots
  serialize :plot_deck
  serialize :plot_discard
  serialize :shadow_deck
  serialize :shadow_discard
  serialize :characters

  aasm do
    state :created, :initial => true
    state :waiting_for_players, :heroes_setup, :sauron_setup, :heroes_turn, :sauron_first_turn, :sauron_turn

    event :wait_for_players do
      transitions :from => :created, :to => :waiting_for_players
    end

    event :heroes_setup do
      transitions :from => [ :created, :waiting_for_players ], :to => :heroes_setup
    end

    event :sauron_setup do
      transitions :from => :heroes_setup, :to => :sauron_setup
    end

    event :sauron_first_turn do
      transitions :from => :sauron_setup, :to => :sauron_first_turn
    end

    event :heroes_turn do
      transitions :from => [ :sauron_turn, :sauron_first_turn ], :to => :heroes_turn
    end

    event :sauron_turn do
      transitions :from => [ :heroes_turn ], :to => :sauron_turn
    end

  end

  def log!( user, actor, action, params= {} )
    actor = actor.is_a?( Integer ) ? actor : actor.id
    logs.create!( board: self, actor_id: actor, user: user, action: action, params: params )
  end

end
