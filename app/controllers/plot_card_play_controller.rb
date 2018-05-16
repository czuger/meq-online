class PlotCardPlayController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor

  def edit
    @plot_cards = @board.current_plots
  end

  def update

  end

end
