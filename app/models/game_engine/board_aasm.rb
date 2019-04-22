module GameEngine
  module BoardAasm

    def self.included(base)
      base.send(:include, AASM)

      base.send(:aasm) do
        state :created, :initial => true
        state :waiting_for_players, :sauron_setup_screen, :edit_sauron_sauron_actions, :hero_draw_cards_screen
        state :hero_rest_screen, :hero_single_hero_draw, :single_hero_rally
        state :hero_movement_screen, :exploration, :play_screen_sauron_plot_cards, :after_rest_advance_story_marker
        state :combat_setup_screen_board_combats, :play_combat_card_screen_board_combats

        event :wait_for_players do
          transitions :from => :created, :to => :waiting_for_players
        end

        event :next_to_sauron_setup_screen do
          transitions :from => [ :created, :waiting_for_players ], :to => :sauron_setup_screen
        end

        event :next_to_edit_sauron_sauron_actions do
          transitions :from => [:sauron_setup_screen, :play_screen_sauron_plot_cards], :to => :edit_sauron_sauron_actions
        end

        event :next_to_hero_single_hero_draw do
          transitions :from => :exploration, :to => :hero_single_hero_draw
        end

        event :next_to_hero_draw_cards_screen do
          transitions :from => :edit_sauron_sauron_actions, :to => :hero_draw_cards_screen
        end

        event :next_to_hero_rest_screen do
          transitions :from => [:hero_draw_cards_screen, :exploration, :hero_single_hero_draw], :to => :hero_rest_screen
        end

        event :next_to_after_rest_advance_story_marker do
          transitions :from => :hero_rest_screen, :to => :after_rest_advance_story_marker
        end

        event :next_to_hero_movement_screen do
          transitions :from => [:play_combat_card_screen_board_combats, :after_rest_advance_story_marker, :exploration, :hero_rest_screen], :to => :hero_movement_screen
        end

        event :next_to_combat_setup_screen_board_combats do
          transitions :from => :hero_movement_screen, :to => :combat_setup_screen_board_combats
        end

        event :next_to_play_combat_card_screen_board_combats do
          transitions :from => :combat_setup_screen_board_combats, :to => :play_combat_card_screen_board_combats
        end

        event :next_to_exploration do
          transitions :from => [:play_combat_card_screen_board_combats, :hero_movement_screen], :to => :exploration
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
