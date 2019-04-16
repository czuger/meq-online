require 'test_helper'

class CombatsControllerTest < ActionDispatch::IntegrationTest
  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )

    @hero = create( :hero, user: @user, board: @board )
    @sauron = create( :sauron, user: @user, board: @board )

    @mob = create( :monster, board: @board )
    @board.create_combat( @hero, @mob )

    @board.aasm_state = 'combat_setup'
    @board.save!

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  test 'should get hero_setup_new' do
    get hero_setup_new_board_combats_url(@board)
    assert_response :success
  end

  test 'should start combat with strength increase' do
    post hero_setup_board_combats_url(@board, button: :increase)

    assert_equal 8, @hero.reload.combat_temporary_strength

    assert_redirected_to play_combat_card_screen_board_combats_url(@board, @hero)
  end

  test 'should start combat with cards drawn' do
    post hero_setup_board_combats_url(@board, button: :draw)

    assert_equal 4, @hero.reload.hand.count

    assert_redirected_to play_combat_card_screen_board_combats_url(@board, @hero)
  end

  # test "should get index" do
  #   get combats_url
  #   assert_response :success
  # end
  #
  # test "should get new" do
  #   get new_combat_url
  #   assert_response :success
  # end
  #
  # test "should create combat" do
  #   assert_difference('Combat.count') do
  #     post combats_url, params: { combat: { board_id: @combat.board_id, hero_cards_played: @combat.hero_cards_played, hero_id: @combat.hero_id, sauron_cards_played: @combat.sauron_cards_played, temporary_strength: @combat.temporary_strength } }
  #   end
  #
  #   assert_redirected_to combat_url(Combat.last)
  # end
  #
  # test "should show combat" do
  #   get combat_url(@combat)
  #   assert_response :success
  # end
  #
  # test "should get edit" do
  #   get edit_combat_url(@combat)
  #   assert_response :success
  # end
  #
  # test "should update combat" do
  #   patch combat_url(@combat), params: { combat: { board_id: @combat.board_id, hero_cards_played: @combat.hero_cards_played, hero_id: @combat.hero_id, sauron_cards_played: @combat.sauron_cards_played, temporary_strength: @combat.temporary_strength } }
  #   assert_redirected_to combat_url(@combat)
  # end
  #
  # test "should destroy combat" do
  #   assert_difference('Combat.count', -1) do
  #     delete combat_url(@combat)
  #   end
  #
  #   assert_redirected_to combats_url
  # end
end
