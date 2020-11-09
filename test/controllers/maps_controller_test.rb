require 'test_helper'

class MapsControllerTest < ActionDispatch::IntegrationTest

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

    connection_for_tests
  end

  test 'should get edit' do
    get edit_map_url(@sauron)
    assert_response :success
  end

end
