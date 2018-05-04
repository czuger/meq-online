require 'test_helper'

class HerosControllerTest < ActionDispatch::IntegrationTest
  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )
    @hero = create( :hero, user: @user, board: @board )

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  test 'should get index' do
    get board_heros_url( @board )
    assert_response :success
  end

  test 'should get join_new' do
    get board_join_url( @board )
    assert_response :success
  end

  test 'should post join_new' do
    post board_join_url( @board )
    assert_redirected_to boards_url
  end

  test 'should POST draw_cards' do
    post board_hero_draw_cards_url( @board, @hero )
    assert_redirected_to board_hero_url(@board, @hero)
  end

  test 'should get heal' do
    get board_hero_heal_url( @board, @hero )
    assert_redirected_to board_hero_url(@board, @hero)
  end

  test 'should get rest' do
    get board_hero_rest_url( @board, @hero )
    assert_redirected_to board_hero_url(@board, @hero)
  end

  test 'should patch take_damages' do
    patch board_hero_take_damages_url( @board, @hero, damage_amount: 3 )
    assert_redirected_to board_hero_url(@board, @hero)
  end

  test 'should POST move' do
    post board_hero_move_url( @board, @hero, params: { move_to: :the_grey_havens, card_used: 1 } )
    assert_redirected_to board_hero_url(@board, @hero)
  end

  # test 'should get new' do
  #   get new_board_hero_url
  #   assert_response :success
  # end

  # test 'should create hero' do
  #   assert_difference('Hero.count') do
  #     post board_heros_url( @board ), params: { hero: { agility: @hero.agility, damage_pool: @hero.damage_pool, fortitude: @hero.fortitude, life_pool: @hero.life_pool, location: @hero.location, name_code: @hero.name_code, rest_pool: @hero.rest_pool, strength: @hero.strength, wisdom: @hero.wisdom } }
  #   end
  #
  #   assert_redirected_to hero_url(Hero.last)
  # end

  test 'should show hero' do
    get board_hero_url( @board, @hero )
    assert_response :success
  end

  # test 'should get edit' do
  #   get edit_hero_url(@hero)
  #   assert_response :success
  # end
  #
  # test 'should update hero' do
  #   patch hero_url(@hero), params: { hero: { agility: @hero.agility, damage_pool: @hero.damage_pool, fortitude: @hero.fortitude, life_pool: @hero.life_pool, location: @hero.location, name_code: @hero.name_code, rest_pool: @hero.rest_pool, strength: @hero.strength, wisdom: @hero.wisdom } }
  #   assert_redirected_to hero_url(@hero)
  # end
  #
  # test 'should destroy hero' do
  #   assert_difference('Hero.count', -1) do
  #     delete hero_url(@hero)
  #   end
  #
  #   assert_redirected_to heros_url
  # end
end
