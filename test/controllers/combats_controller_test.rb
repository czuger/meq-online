require 'test_helper'

class CombatsControllerTest < ActionDispatch::IntegrationTest
  setup do
    @combat = combats(:one)
  end

  test "should get index" do
    get combats_url
    assert_response :success
  end

  test "should get new" do
    get new_combat_url
    assert_response :success
  end

  test "should create combat" do
    assert_difference('Combat.count') do
      post combats_url, params: { combat: { board_id: @combat.board_id, hero_cards_played: @combat.hero_cards_played, hero_id: @combat.hero_id, sauron_cards_played: @combat.sauron_cards_played, temporary_strength: @combat.temporary_strength } }
    end

    assert_redirected_to combat_url(Combat.last)
  end

  test "should show combat" do
    get combat_url(@combat)
    assert_response :success
  end

  test "should get edit" do
    get edit_combat_url(@combat)
    assert_response :success
  end

  test "should update combat" do
    patch combat_url(@combat), params: { combat: { board_id: @combat.board_id, hero_cards_played: @combat.hero_cards_played, hero_id: @combat.hero_id, sauron_cards_played: @combat.sauron_cards_played, temporary_strength: @combat.temporary_strength } }
    assert_redirected_to combat_url(@combat)
  end

  test "should destroy combat" do
    assert_difference('Combat.count', -1) do
      delete combat_url(@combat)
    end

    assert_redirected_to combats_url
  end
end
