require 'test_helper'

class MapCoordinatesControllerTest < ActionDispatch::IntegrationTest
  test "should get edit" do
    get map_coordinates_edit_url
    assert_response :success
  end

  test "should get update" do
    get map_coordinates_update_url
    assert_response :success
  end

end
