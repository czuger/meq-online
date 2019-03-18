require 'test_helper'

class EventsControllerControllerTest < ActionDispatch::IntegrationTest
  test "should get edit" do
    get events_controller_edit_url
    assert_response :success
  end

  test "should get update" do
    get events_controller_update_url
    assert_response :success
  end

end
