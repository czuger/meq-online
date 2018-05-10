class ApplicationController < ActionController::Base

  def require_logged_in
    redirect_to new_sessions_path unless current_user
  end

  private

  def current_user
    @current_user ||= ( User.find(session[:user_id]) if session[:user_id] )
  end

  def current_user?( hero_or_sauron )
    current_user.id == hero_or_sauron.user_id
  end

  helper_method :current_user

  # Security methods
  def set_actor_ensure_actor
    @actor = Actor.find(params[:id])
    @board = @actor.board
    ensure_board
  end

  def set_actor_ensure_board
    @actor = Actor.find(params[:id])
    @board = @actor.board
    ensure_board
  end

  def ensure_sauron
    raise "Sauron #{@sauron.inspect} is not owned by #{current_user.inspect}" unless @sauron.user_id == current_user.id
  end

  def ensure_sauron
    raise "Sauron #{@sauron.inspect} is not owned by #{current_user.inspect}" unless @sauron.user_id == current_user.id
  end

  def ensure_board
    raise "Board #{@board.inspect} can't be modified by #{current_user.id}" unless @board.users.pluck(:id).include?(current_user.id)
  end

end
