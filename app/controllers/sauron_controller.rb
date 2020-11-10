class SauronController < ApplicationController
  before_action :require_logged_in
  before_action :set_sauron_ensure_sauron

  def show
  end

  def setup_screen
    @plot_card = GameData::Plots.new.get(@board.current_plots.first.plot_card)
  end

  def setup_finished
    @board.transaction do
      GameData::Objectives.set_objectives @board

      GameData::Events.new.draw_next_event_card(@board)

      @board.next_to_edit_sauron_sauron_actions!

      @board.set_sauron_activation_state( true )
    end

    redirect_to edit_sauron_sauron_actions_path(@actor)
  end

end
