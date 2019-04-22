require 'test_helper'

class SauronActionsControllerTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )
    @hero = create( :hero, user: @user, board: @board )
    @sauron = create( :sauron, user: @user, board: @board )
    @board_message = create( :board_message, sender: @sauron, reciever: @hero )
    @board.users << @user

    @board.aasm_state = 'edit_sauron_sauron_actions'
    @board.save!

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  test 'should get edit' do
    get edit_sauron_sauron_actions_url(@sauron)
    assert_response :success
  end

  test 'should update action' do
    patch sauron_sauron_actions_url(@sauron), params: { actions: [ :place_influence_1 ] }
    assert_response :success
  end

  test 'should set influence' do
    post set_influence_sauron_sauron_actions_url(@sauron), params: { locations: { the_shire: 3 } }
    assert_response :success
  end
end
