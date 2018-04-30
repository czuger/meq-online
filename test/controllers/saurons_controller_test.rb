require 'test_helper'

class SauronsControllerTest < ActionDispatch::IntegrationTest
  setup do
    @sauron = saurons(:one)
  end

  test "should get index" do
    get saurons_url
    assert_response :success
  end

  test "should get new" do
    get new_sauron_url
    assert_response :success
  end

  test "should create sauron" do
    assert_difference('Sauron.count') do
      post saurons_url, params: { sauron: { board_id: @sauron.board_id, plot_cards: @sauron.plot_cards, shadow_cards: @sauron.shadow_cards, user_id: @sauron.user_id } }
    end

    assert_redirected_to sauron_url(Sauron.last)
  end

  test "should show sauron" do
    get sauron_url(@sauron)
    assert_response :success
  end

  test "should get edit" do
    get edit_sauron_url(@sauron)
    assert_response :success
  end

  test "should update sauron" do
    patch sauron_url(@sauron), params: { sauron: { board_id: @sauron.board_id, plot_cards: @sauron.plot_cards, shadow_cards: @sauron.shadow_cards, user_id: @sauron.user_id } }
    assert_redirected_to sauron_url(@sauron)
  end

  test "should destroy sauron" do
    assert_difference('Sauron.count', -1) do
      delete sauron_url(@sauron)
    end

    assert_redirected_to saurons_url
  end
end
