require 'test_helper'

class EventsControllerControllerTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )

    @sauron = create( :sauron, user: @user, board: @board )
    @board.users << @user

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  test 'should get edit' do
    get edit_event_url( @sauron )
    assert_response :success
  end

  test 'should get update' do
    patch event_url( @sauron ), params: {}
    assert_response :success
  end

end
