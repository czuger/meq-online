class ApplicationController < ActionController::Base

  def require_logged_in
    redirect_to new_sessions_path unless current_user
  end
  #
  # private

  def current_user
    @current_user ||= ( User.find(session[:user_id]) if session[:user_id] )
  end

  def current_user?( hero_or_sauron )
    current_user.id == hero_or_sauron.user_id
  end

  helper_method :current_user

  # Security methods


  # For private actor part
  def set_actor_ensure_actor
    @actor = Actor.find(actor_from_params)
    @board = @actor.board
    ensure_board
  end

  # For board that could be accessed by all actors
  def set_actor_ensure_board
    @actor = Actor.find(actor_from_params)
    @board = @actor.board
    ensure_board
  end

  def ensure_actor
    raise "#{@actor.inspect} is not owned by #{current_user.inspect}" unless @actor.user_id == current_user.id
  end

  def ensure_board
    raise "Board #{@board.inspect} can't be modified by #{current_user.id}" unless @board.users.pluck(:id).include?(current_user.id)
  end

  def actor_from_params
    params[:id] || params[:hero_id] || params[:sauron_id]
  end

end