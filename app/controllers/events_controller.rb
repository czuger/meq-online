class EventsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_actor

  def edit
  end

  def update
  end

  def finished
    @board.transaction do
      @board.next_to_sauron_actions!

      redirect_to edit_sauron_action_path(@actor)
    end
  end

end
