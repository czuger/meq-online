require 'test_helper'

class BoardMessagesControllerTest < ActionDispatch::IntegrationTest
  setup do
    @board_message = board_messages(:one)
  end

  test "should get index" do
    get board_messages_url
    assert_response :success
  end

  test "should get new" do
    get new_board_message_url
    assert_response :success
  end

  test "should create board_message" do
    assert_difference('BoardMessage.count') do
      post board_messages_url, params: { board_message: { board_id: @board_message.board_id, reciever: @board_message.reciever, sender: @board_message.sender, text: @board_message.text } }
    end

    assert_redirected_to board_message_url(BoardMessage.last)
  end

  test "should show board_message" do
    get board_message_url(@board_message)
    assert_response :success
  end

  test "should get edit" do
    get edit_board_message_url(@board_message)
    assert_response :success
  end

  test "should update board_message" do
    patch board_message_url(@board_message), params: { board_message: { board_id: @board_message.board_id, reciever: @board_message.reciever, sender: @board_message.sender, text: @board_message.text } }
    assert_redirected_to board_message_url(@board_message)
  end

  test "should destroy board_message" do
    assert_difference('BoardMessage.count', -1) do
      delete board_message_url(@board_message)
    end

    assert_redirected_to board_messages_url
  end
end
