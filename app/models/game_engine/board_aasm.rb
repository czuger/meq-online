module GameEngine
  module BoardAasm

    def self.included(base)
      base.send(:include, AASM)

      base.send(:aasm) do
        state :created, :initial => true
        state :waiting_for_players, :sauron_setup_screen, :sauron_actions, :hero_draw_cards_screen
        state :rest_step, :single_hero_draw, :single_hero_rally
        state :movement, :exploration, :play_screen_sauron_plot_cards, :after_rest_advance_story_marker
        state :combat_setup, :play_combat_card

        event :wait_for_players do
          transitions :from => :created, :to => :waiting_for_players
        end

        event :next_to_sauron_setup_screen do
          transitions :from => [ :created, :waiting_for_players ], :to => :sauron_setup_screen
        end

        event :next_to_sauron_actions do
          transitions :from => [:sauron_setup_screen, :play_screen_sauron_plot_cards], :to => :sauron_actions
        end

        event :next_to_single_hero_draw do
          transitions :from => :exploration, :to => :single_hero_draw
        end

        event :next_to_hero_draw_cards_screen do
          transitions :from => :sauron_actions, :to => :hero_draw_cards_screen
        end

        event :next_to_rest_step do
          transitions :from => [:hero_draw_cards_screen, :exploration, :single_hero_draw], :to => :rest_step
        end

        event :next_to_after_rest_advance_story_marker do
          transitions :from => :rest_step, :to => :after_rest_advance_story_marker
        end

        event :next_to_movement do
          transitions :from => [:after_rest_advance_story_marker, :exploration, :rest_step], :to => :movement
        end

        event :next_to_combat_setup do
          transitions :from => :movement, :to => :combat_setup
        end

        event :next_to_play_combat_card do
          transitions :from => :combat_setup, :to => :play_combat_card
        end

        event :next_to_exploration do
          transitions :from => :movement, :to => :exploration
        end

        event :next_to_play_screen_sauron_plot_cards do
          transitions :from => :exploration, :to => :play_screen_sauron_plot_cards
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
