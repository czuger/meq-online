require 'test_helper'

class BoardStepsControllerTest < ActionDispatch::IntegrationTest
  test "should get edit" do
    get board_steps_edit_url
    assert_response :success
  end

  test "should get update" do
    get board_steps_update_url
    assert_response :success
  end

end
