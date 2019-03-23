module GameEngine
  module BoardAasm

    def self.included(base)
      base.send(:include, AASM)

      base.send(:aasm) do
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
    end

    def all_heroes_played?
      heroes.pluck( :turn_finished ).each do |tf|
        return false unless tf
      end
      true
    end

    def clean_heroes_played_status!
      heroes.update_all( turn_finished: false )
    end

  end
end
