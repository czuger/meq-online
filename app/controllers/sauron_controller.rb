class SauronController < ApplicationController
  before_action :require_logged_in
  before_action :set_actor_ensure_actor

  def show
  end

  def setup_screen
    @plot_card = GameData::Plots.new.get(@board.current_plots.first.plot_card)
  end

  def setup_finished
    @board.transaction do
      GameData::Objectives.set_objectives @board

      GameData::Events.new.draw_next_event_card(@board)

      @board.next_to_edit_sauron_action!

      @board.set_sauron_activation_state( true )
    end

    redirect_to edit_sauron_action_path(@actor)
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
      @board.activate_current_hero
      @board.save!

      redirect_to boards_path
    end
  end

end
