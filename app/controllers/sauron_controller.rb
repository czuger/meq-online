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
      @board.next_to_sauron_actions!
    end

    # Should lead to event draw
    GameEngine::RedirectFromBoardState.redirect(@board, @actor ){ |r| redirect_to r }
  end

  def draw_event_card
  end

  def choose_event_card
  end

  def execute_event_card
  end

  def event_card_step_finished
  end

end
