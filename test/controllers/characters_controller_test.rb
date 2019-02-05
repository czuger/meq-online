require 'test_helper'

class CharactersControllerTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )
    @hero = create( :hero, user: @user, board: @board )
    @board.users << @user

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  teardown do
    OmniAuth.config.test_mode = false
  end

  test 'should get edit' do
    get edit_character_url @hero
    assert_response :success
  end

  test 'should get update' do
    put character_url @hero
    assert_redirected_to edit_character_url @hero
  end

end
