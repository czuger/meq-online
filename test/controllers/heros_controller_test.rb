require 'test_helper'

class HerosControllerTest < ActionDispatch::IntegrationTest
  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board, favors: [ :the_shire, :old_forest ] )
    @hero = create( :hero, user: @user, board: @board, location: :the_shire )
    @sauron = create( :sauron, user: @user, board: @board )
    @board.users << @user

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!

    # pp User.all
  end

  test 'should get join_new' do
    get board_join_url( @board )
    assert_response :success
  end

  test 'should post join_new' do
    post board_join_url( @board )
    assert_redirected_to boards_url
  end

  test 'should show hero' do
    get hero_url( @hero )
    assert_response :success
  end

  test 'should finish movement' do
    @board.aasm_state = 'hero_movement_screen'
    @board.save!

    get hero_movement_finished_url( @hero )
    assert_redirected_to hero_exploration_screen_url( @hero )
  end

  test 'should rest and be redirected to advance story marker screen' do
    @board.aasm_state = :hero_rest_screen
    @board.save!

    get hero_rest_rest_url( @hero )
    assert_redirected_to hero_after_rest_advance_story_marker_screen_url(@hero)
  end

  test 'should heal and be redirected to movement' do
    @board.aasm_state = :hero_rest_screen
    @board.story_marker_corruption = 5
    @board.story_marker_ring = 6
    @board.save!

    assert_difference '@board.reload.story_marker_conquest' do
      assert_no_difference '@board.reload.story_marker_corruption' do
        assert_no_difference '@board.reload.story_marker_ring' do
          get hero_rest_heal_url( @hero )
        end
      end
    end

    assert_redirected_to hero_movement_screen_url(@hero)
  end

  test 'should get after_rest_advance_story_marker_screen' do
    @board.story_marker_ring = 6
    @board.save!

    get hero_after_rest_advance_story_marker_screen_url( @hero )
    assert_response :success

    # puts @response.body

    assert_select 'a', 'Conquest'
    assert_select 'a', 'Corruption'
  end

  test 'should advance selected marker' do
    @board.aasm_state = :hero_rest_screen
    @board.save!

    assert_difference '@board.reload.story_marker_conquest' do
      get hero_after_rest_advance_story_marker_url( @hero, marker: 'Conquest' )
    end

    assert_redirected_to hero_movement_screen_url(@hero)
  end

  # test 'should patch take_damages' do
  #   patch hero_take_damages_url( @hero, damage_amount: 3 )
  #   assert_redirected_to hero_url(@hero)
  # end

  test 'should get a favor' do
    post hero_explore_url( @hero, tokens: { favor: [ :favor ] } )
    assert_redirected_to hero_exploration_screen_url(@hero)
    assert_equal 1, @hero.reload.favor
  end

  test 'should get only one favor and leave one' do
    @board.favors << :the_shire
    @board.save

    post hero_explore_url( @hero, tokens: { favor: [ :favor ] } )
    assert_redirected_to hero_exploration_screen_url(@hero)
    assert_equal 1, @hero.reload.favor

    assert_equal %w( the_shire old_forest ).sort, @board.reload.favors.sort
  end

  test 'should get two favors and leave none' do
    @board.favors << :the_shire
    @board.save

    post hero_explore_url( @hero, tokens: { favor: [ :favor, :favor ] } )
    assert_redirected_to hero_exploration_screen_url(@hero)
    assert_equal 2, @hero.reload.favor

    assert_equal %w( old_forest ).sort, @board.reload.favors.sort
  end

  test 'after movement, should create a combat' do
    @board.aasm_state = :hero_movement_screen
    @board.save!

    mob = @board.create_monster( :agent, :old_forest, :monsters_pool_orange )

    assert_difference 'Combat.count' do
      post hero_move_url( @hero, params: { button: :old_forest, selected_cards: @hero.hand.first } )
    end

    assert @board.combat

    assert_redirected_to combat_setup_screen_board_combats_url(@board)
  end

  # test 'should POST draw_cards' do
  #   post hero_draw_cards_url( @hero )
  #   assert_redirected_to hero_url(@hero)
  # end

  # test 'should get heal' do
  #   get hero_heal_url( @hero )
  #   assert_redirected_to hero_url(@hero)
  # end

  # test 'should get rest' do
  #   get hero_rest_url( @hero )
  #   assert_redirected_to hero_url(@hero)
  # end

  # test 'should POST move' do
  #   post hero_move_url( @hero, params: { move_to: :the_grey_havens, card_used: 1 } )
  #   assert_redirected_to hero_url(@hero)
  # end

  # test 'should get new' do
  #   get new_hero_url
  #   assert_response :success
  # end

  # test 'should create hero' do
  #   assert_difference('Hero.count') do
  #     post board_heros_url( @board ), params: { hero: { agility: @hero.agility, damage_pool: @hero.damage_pool, fortitude: @hero.fortitude, life_pool: @hero.life_pool, location: @hero.location, name_code: @hero.name_code, rest_pool: @hero.rest_pool, strength: @hero.strength, wisdom: @hero.wisdom } }
  #   end
  #
  #   assert_redirected_to hero_url(Hero.last)
  # end

  # test 'should show hero' do
  #   get hero_url( @hero )
  #   assert_response :success
  # end

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
