require 'test_helper'

class MapCoordinatesControllerTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )
    @board.users << @user
    @sauron = create( :sauron, board: @board, user: @user )

    @board.aasm_state = 'edit_sauron_sauron_actions'
    @board.save!

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    post '/auth/google_oauth2'
    follow_redirect!
  end

  test 'should get edit' do
    get map_coordinates_edit_url
    assert_response :success
  end
  #
  # test "should get update" do
  #   get map_coordinates_update_url
  #   assert_response :success
  # end

end
