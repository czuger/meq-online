require 'test_helper'

class BoardStepsControllerTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )
    @sauron = create( :sauron, user: @user, board: @board )
    @board.users << @user

    @board.aasm_state = 'edit_sauron_action'
    @board.save!

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  teardown do
    OmniAuth.config.test_mode = false
  end

  test 'should get edit' do
    get edit_board_step_url @sauron
    assert_response :success
  end

  # test 'should get update' do
  #   put board_step_url @sauron, next_event: :wait_for_players
  #   assert_redirected_to edit_board_step_url @sauron
  # end

end
