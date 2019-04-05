module GameEngine
  module BoardAasm

    def self.included(base)
      base.send(:include, AASM)

      base.send(:aasm) do
        state :created, :initial => true
        state :waiting_for_players, :sauron_setup, :sauron_actions, :heroes_draw_cards
        state :rest_step, :single_hero_draw, :single_hero_rally
        state :movement, :exploration, :plot

        event :wait_for_players do
          transitions :from => :created, :to => :waiting_for_players
        end

        event :next_to_sauron_setup do
          transitions :from => [ :created, :waiting_for_players ], :to => :sauron_setup
        end

        event :next_to_sauron_actions do
          transitions :from => [:sauron_setup, :plot], :to => :sauron_actions
        end

        event :next_to_single_hero_draw do
          transitions :from => :exploration, :to => :single_hero_draw
        end

        event :next_to_heroes_draw_cards do
          transitions :from => :sauron_actions, :to => :heroes_draw_cards
        end

        event :next_to_rest_step do
          transitions :from => [:heroes_draw_cards, :exploration], :to => :rest_step
        end

        event :next_to_movement do
          transitions :from => [:exploration, :rest_step], :to => :movement
        end

        event :next_to_exploration do
          transitions :from => :movement, :to => :exploration
        end

        event :next_to_plot do
          transitions :from => :exploration, :to => :plot
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
