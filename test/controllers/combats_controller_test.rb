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

    @board.users << @user

    @board.aasm_state = 'combat_setup_screen_board_combats'
    @board.save!

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  test 'should get hero_setup_new' do
    get combat_setup_screen_board_combats_url(@board)
    assert_response :success
  end

  test 'should start combat with strength increase' do
    post combat_setup_board_combats_url(@board, button: :increase)

    assert_equal 8, @board.combat.reload.temporary_hero_strength

    assert_redirected_to play_combat_card_screen_board_combats_url(@board, @hero)
  end

  test 'hero should show play card screen' do
    get play_combat_card_screen_board_combats_url(@board, @hero)
    assert_response :success
  end

  test 'mob should show play card screen' do
    get play_combat_card_screen_board_combats_url(@board, @hero)
    assert_response :success
  end

  test 'hero should play card' do
    post play_combat_card_hero_board_combats_url(@board, selected_card: 1)
    assert_redirected_to boards_url

    follow_redirect!
  end

  test 'mob should play card' do
    post play_combat_card_mob_board_combats_url(@board, selected_card: 1)
    assert_redirected_to boards_url
  end

end
