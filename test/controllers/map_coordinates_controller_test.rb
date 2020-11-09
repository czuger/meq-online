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

    connection_for_tests
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
