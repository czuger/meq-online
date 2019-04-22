require 'test_helper'

class CombatsControllerTest < ActionDispatch::IntegrationTest
  setup do
    OmniAuth.config.test_mode = true

    @game_data_heroes ||= GameData::Heroes.new
    @game_data_mobs_cards ||= GameData::MobsCards.new

    @user = create( :user )
    @board = create( :board )

    @hero = create( :hero, user: @user, board: @board, hand: @game_data_heroes.get_deck(:argalad ) )
    @sauron = create( :sauron, user: @user, board: @board )

    @mob = create( :monster, board: @board, hand: @game_data_mobs_cards.get_deck( 'ravager' ) )
    @board.create_combat( @hero, @mob )

    @board.combat.temporary_hero_strength = 5
    @board.combat.save!

    @board.users << @user

    @board.aasm_state = 'combat_setup_screen_board_combats'
    @board.current_hero = @hero
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
    post play_combat_card_hero_board_combats_url(@board, selected_card: @hero.hand.sample)
    assert_redirected_to boards_url

    follow_redirect!
  end

  test 'combat_should be terminated' do
    @mob.life = 0
    @mob.save!

    @board.aasm_state = 'play_combat_card_screen_board_combats'
    @board.save!

    get terminate_board_combats_url(@board)
    assert_redirected_to hero_movement_screen_path(@hero)
  end

  test 'mob should play card' do
    post play_combat_card_mob_board_combats_url(@board, selected_card: @mob.hand.sample)
    assert_redirected_to boards_url
  end

  test 'if hero and mob play, should resolve combat' do

    post play_combat_card_hero_board_combats_url(@board, selected_card: @hero.hand.sample)
    assert_redirected_to boards_url

    assert_difference 'CombatCardPlayed.count', 2 do
      post play_combat_card_mob_board_combats_url(@board, selected_card: @mob.hand.sample)
    end
    assert_redirected_to board_combats_url(@board)

    follow_redirect!
  end

end
