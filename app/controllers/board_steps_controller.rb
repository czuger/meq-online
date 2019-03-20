class BoardStepsController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_board, only: [:edit,:update]

  def edit
  end

  def update
    next_event = params['next_event'].to_sym

    # Don't use @board.send( "may_#{next_event}?" ) to check if event is good, it is far too dangerous
    available_events = @board.aasm.events(:permitted => true).map(&:name)
    raise "Forbidden event : #{next_event} not in #{available_events.inspect}" unless available_events.include?(next_event)

    old_state = @board.aasm_state
    @board.transaction do
      @board.send( "#{next_event}!" )
      @board.log( @actor, :switch_state, old_state: old_state.humanize, new_state: @board.aasm_state.humanize )
    end

    redirect_to edit_board_step_path(@actor)
  end

end
