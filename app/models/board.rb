class Board < ApplicationRecord

  include AASM
  include BoardAasm

  has_many :logs, dependent: :destroy

  has_many :heroes, dependent: :destroy
  has_one :sauron, dependent: :destroy

  has_and_belongs_to_many :users

  belongs_to :current_hero, class_name: 'Hero', optional: true

  aasm do
    state :created, :initial => true
    state :waiting_for_players, :sauron_setup, :event_step, :sauron_actions, :heroes_draw_cards
    state :play_shadow_card_at_start_of_hero_turn, :rest_step

    event :wait_for_players do
      transitions :from => :created, :to => :waiting_for_players
    end

    event :next_to_sauron_setup do
      transitions :from => [ :created, :waiting_for_players ], :to => :sauron_setup
    end

    event :next_to_event_step do
      transitions :from => :sauron_setup, :to => :event_step
    end

    event :next_to_sauron_actions do
      transitions :from => :event_step, :to => :sauron_actions
    end

    event :next_to_heroes_draw_cards do
      transitions :from => :sauron_actions, :to => :heroes_draw_cards
    end

    event :next_to_play_shadow_card_at_start_of_hero_turn do
      transitions :from => :heroes_draw_cards, :to => :play_shadow_card_at_start_of_hero_turn
    end

    event :next_to_rest_step do
      transitions :from => :play_shadow_card_at_start_of_hero_turn, :to => :rest_step
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

  #
  # Activation state methode
  #
  def heroes_actives?
    heroes.map{ |e| e.active }.inject(:|)
  end

  def set_hero_activation_state( hero, active= false )
    hero.active = active
    hero.save!
  end

  def set_heroes_activation_state( active= false )
    heroes.each do |h|
      h.active = active
      h.save!
    end
  end

  def set_sauron_activation_state( active= false )
    sauron.active = active
    sauron.save!
  end

  def set_all_actors_activation_state( active= false )
    set_sauron_activation_state active
    set_heroes_activation_state active
  end

  #
  # Log methodes
  #
  def log( actor, action, params= {} )
    logs.create!( board: self, actor: actor, action: action, params: params )
  end

end
