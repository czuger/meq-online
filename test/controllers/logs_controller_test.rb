require 'test_helper'

class LogsControllerTest < ActionDispatch::IntegrationTest
  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )
    @hero = create( :hero, board: @board, user: @user )
    p @hero
    @log = create(:log, board: @board, user: @user, player: @hero )

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  test "should get index" do
    get board_logs_url( @board )
    assert_response :success
  end

  # test "should get new" do
  #   get new_log_url
  #   assert_response :success
  # end
  #
  # test "should create log" do
  #   assert_difference('Log.count') do
  #     post logs_url, params: { log: { action: @log.action, board_id: @log.board_id, params: @log.params } }
  #   end
  #
  #   assert_redirected_to log_url(Log.last)
  # end
  #
  # test "should show log" do
  #   get log_url(@log)
  #   assert_response :success
  # end
  #
  # test "should get edit" do
  #   get edit_log_url(@log)
  #   assert_response :success
  # end
  #
  # test "should update log" do
  #   patch log_url(@log), params: { log: { action: @log.action, board_id: @log.board_id, params: @log.params } }
  #   assert_redirected_to log_url(@log)
  # end
  #
  # test "should destroy log" do
  #   assert_difference('Log.count', -1) do
  #     delete log_url(@log)
  #   end
  #
  #   assert_redirected_to logs_url
  # end
end
