class SauronController < ApplicationController
  before_action :require_logged_in
  before_action :set_actor_ensure_actor

  def show
  end

  def setup
    @plot_card = GameData::StartingPlots.new.get(@board.current_plots['plot-card-1'])
  end

  def setup_finished
    @board.transaction do
      GameData::Objectives.set_objectives @board

      events = GameData::Events.new

      events.set_random_card @board
      events.place_characters_and_influence @board, @board.last_event_card

      @board.next_to_event_step!
      @board.next_to_sauron_actions!

      @board.set_sauron_activation_state( true )
    end

    redirect_to edit_sauron_action_path(@actor)
  end

  def choose_event_card_screen
  end

  def choose_event_card
  end

  def execute_event_card_screen
  end

  def execute_event_card
  end

  #
  # Mouvement break schedule methods
  #
  def movement_break_schedule_screen
  end

  def movement_break_schedule_add
  end

  def movement_break_schedule_finished
    @board.transaction do
      @board.next_to_movement!
      @board.switch_to_current_hero
      @board.save!

      redirect_to boards_path
    end
  end

end
