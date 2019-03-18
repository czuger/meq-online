class SauronController < ApplicationController
  before_action :require_logged_in
  before_action :set_actor_ensure_actor

  def show
    GameEngine::RedirectFromBoardState.redirect(@board, @actor ){ |r| redirect_to r }
  end

  def setup
    @plot_card = GameData::StartingPlots.new.get(@board.current_plots['plot-card-1'])
  end

  def setup_finished
    @board.transaction do
      GameData::Objectives.set_objectives @board

      GameData::Events.new.set_random_card @board

      @board.set_all_actors_activation_state( true )

      @board.next_to_event_state!
    end

    redirect_to edit_event_path(@actor)
  end

  def choose_event_card_screen
  end

  def choose_event_card
  end

  def execute_event_card_screen
  end

  def execute_event_card
  end

end
