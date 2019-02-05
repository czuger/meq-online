class SessionsController < ApplicationController

  skip_before_action :verify_authenticity_token, if: -> { Rails.env.development? || Rails.env.test? }

  def new
  end

  def create
    ah = auth_hash

    @user = User.find_or_create_by!(provider: ah.provider, uid: ah.uid) do |user|
      user.name = ah.info.name
      user.email = ah.info.email
    end

    session[:user_id] = @user.id

    redirect_to boards_path
  end

  def failure
    redirect_to new_sessions_path, alert: "Authentication failed, please try again."
  end

  def destroy
    session[:user_id] = nil
    redirect_to root_url
  end

  protected

  def auth_hash
    request.env['omniauth.auth']
  end

end