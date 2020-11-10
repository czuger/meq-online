require 'test_helper'

class ShadowPoolControllerTest < ActionDispatch::IntegrationTest
  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )
    @hero = create( :hero, user: @user, board: @board )
    @sauron = create( :sauron, user: @user, board: @board )
    # @board_message = create( :board_message, sender: @sauron, reciever: @hero )
    @board.users << @user

    @board.aasm_state = 'edit_sauron_sauron_actions'
    @board.save!

    connection_for_tests
  end

  test 'should set influence' do
    patch shadow_pools_update_from_map_url(@sauron), params: { current_val: 7 }
    assert_response :success
  end
end
