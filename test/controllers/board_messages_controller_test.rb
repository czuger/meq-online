require 'test_helper'

class BoardMessagesControllerTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )
    @hero = create( :hero, user: @user, board: @board )
    @sauron = create( :sauron, user: @user, board: @board )
    @board_message = create( :board_message, sender: @sauron, reciever: @hero )
    @board.users << @user

    @board.aasm_state = 'sauron_actions'
    @board.save!

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  test 'should get index' do
    get actor_board_messages_url(@sauron)
    assert_response :success
  end
  #
  # test 'should get new' do
  #   get new_board_message_url
  #   assert_response :success
  # end

  test 'should create board_message' do
    assert_difference('BoardMessage.count') do
      post actor_board_messages_url(@sauron), params: { board_message: { reciever_id: @board_message.reciever_id, text: @board_message.text } }
    end

    assert_redirected_to actor_board_messages_url(@sauron)
  end

  # test 'should show board_message' do
  #   get board_message_url(@board_message)
  #   assert_response :success
  # end

  # test 'should get edit' do
  #   get edit_board_message_url(@board_message)
  #   assert_response :success
  # end
  #
  # test 'should update board_message' do
  #   patch board_message_url(@board_message), params: { board_message: { board_id: @board_message.board_id, reciever: @board_message.reciever, sender: @board_message.sender, text: @board_message.text } }
  #   assert_redirected_to board_message_url(@board_message)
  # end

  # test 'should destroy board_message' do
  #   assert_difference('BoardMessage.count', -1) do
  #     delete board_message_url(@board_message)
  #   end
  #
  #   assert_redirected_to board_messages_url
  # end
end
