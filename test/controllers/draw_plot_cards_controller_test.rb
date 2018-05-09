require 'test_helper'

class DrawPlotCardsControllerTest < ActionDispatch::IntegrationTest
  test "should get draw_cards" do
    get draw_plot_cards_draw_cards_url
    assert_response :success
  end

  test "should get edit" do
    get draw_plot_cards_edit_url
    assert_response :success
  end

  test "should get update" do
    get draw_plot_cards_update_url
    assert_response :success
  end

end
