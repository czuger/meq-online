require 'test_helper'

class PlotCardPlayControllerTest < ActionDispatch::IntegrationTest
  test "should get edit" do
    get plot_card_play_edit_url
    assert_response :success
  end

  test "should get update" do
    get plot_card_play_update_url
    assert_response :success
  end

end
