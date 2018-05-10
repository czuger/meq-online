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

  def ensure_sauron
    raise "Sauron #{@sauron.inspect} is not owned by #{current_user.inspect}" unless @sauron.user_id == current_user.id
  end

  helper_method :current_user

end
