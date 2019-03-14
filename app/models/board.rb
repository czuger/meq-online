class Board < ApplicationRecord

  include AASM
  include BoardAasm

  has_many :logs, dependent: :destroy

  has_many :heroes, dependent: :destroy
  has_one :sauron, dependent: :destroy

  has_and_belongs_to_many :users

  aasm do
    state :created, :initial => true
    state :waiting_for_players, :sauron_setup, :sauron_actions

    event :wait_for_players do
      transitions :from => :created, :to => :waiting_for_players
    end

    event :next_to_sauron_setup do
      transitions :from => [ :created, :waiting_for_players ], :to => :sauron_setup
    end

    event :next_to_sauron_actions do
      transitions :from => :sauron_setup, :to => :sauron_actions
    end

    # event :back_to_sauron_first_turn do
    #   transitions :from => [ :heroes_turn ], :to => :sauron_first_turn
    # end

    event :next_to_heroes_turn do
      transitions :from => [ :sauron_turn, :sauron_first_turn ], :to => :heroes_turn
    end

    event :back_to_heroes_turn do
      transitions :from => [ :sauron_turn ], :to => :heroes_turn
    end

    event :next_to_sauron_turn do
      transitions :from => [ :heroes_turn ], :to => [ :sauron_turn, :heroes_turn ], :guard => :all_heroes_played?, :after => Proc.new { clean_heroes_played_status! }
    end

    event :back_to_sauron_turn do
      transitions :from => [ :heroes_turn ], :to => :sauron_turn
    end

  end

  def log!( user, actor, action, params= {} )
    actor = actor.is_a?( Integer ) ? actor : actor.id
    logs.create!( board: self, actor_id: actor, user: user, action: action, params: params )
  end

end
