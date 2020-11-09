require 'test_helper'

class HerosControllerTest < ActionDispatch::IntegrationTest
  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board, favors: [ :old_forest, :bree ] )
    @hero = create( :hero, user: @user, board: @board, location: :bree )
    @sauron = create( :sauron, user: @user, board: @board )
    @board.users << @user

    @board.current_hero = @hero
    @board.save!

    connection_for_tests

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
    assert_redirected_to hero_exploration_url( @hero )
  end

  test 'should finish drawing cards and be redirected to boards_path' do
    @board.aasm_state = 'hero_draw_cards_screen'
    @board.save!

    get hero_draw_cards_finished_url( @hero )
    assert_redirected_to boards_path
    follow_redirect!

    assert_select 'td', 'Sauron'
    assert_select "a[href=?]", hero_rest_screen_path(@hero)

    get hero_rest_screen_url( @hero )
    assert_select 'a[href=?]', hero_rest_rest_path(@hero)
  end

  test 'should finish drawing cards and be redirected to boards_path in movement state' do
    @board.aasm_state = 'hero_draw_cards_screen'
    @board.save!

    @hero.location = 'barad_dur'
    @hero.save!

    get hero_draw_cards_finished_url( @hero )
    assert_redirected_to boards_path
    follow_redirect!

    assert_select 'td', 'Sauron'
    # puts @response.body
    assert_select "a[href=?]", hero_movement_screen_path(@hero)
  end

  test 'should finish drawing cards and be rest screen. Only rest button should be available.' do
    @board.aasm_state = 'hero_draw_cards_screen'
    @board.save!

    @hero.location = 'rivendell'
    @hero.save!

    get hero_draw_cards_finished_url( @hero )
    assert_redirected_to boards_path
    follow_redirect!

    assert_select 'td', 'Sauron'
    assert_select 'a[href=?]', hero_rest_screen_path(@hero)

    get hero_rest_screen_url( @hero )
    assert_select 'a[href=?]', hero_rest_rest_path(@hero)
    assert_select 'a[href=?]', hero_rest_heal_path(@hero)
  end

  test 'should finish drawing cards and be redirected to boards_path but prepared for combat' do
    @board.aasm_state = 'hero_draw_cards_screen'
    @board.save!

    create( :cave_troll, location: @hero.location, board: @board )

    get hero_draw_cards_finished_url( @hero )
    assert_redirected_to boards_path
    follow_redirect!

    # puts @response.body

    assert_select 'td', 'Sauron'
    assert_select "a[href=?]", combat_setup_screen_board_combats_path(@board)
  end


  test 'should finish drawing cards and be redirected to boards_path if we have 2 heroes' do
    @board.aasm_state = 'hero_draw_cards_screen'
    @board.save!

    @eometh = create( :eometh, board: @board, user: @user )

    get hero_draw_cards_finished_url( @hero )
    assert_redirected_to boards_path
    follow_redirect!

    assert_select 'td', 'Sauron'
    assert_select "a[href=?]", hero_draw_cards_screen_path(@eometh)
  end

  test 'should show rest screen and not show a link to discard corruption card' do
    @board.aasm_state = :hero_rest_screen
    @board.save!

    get hero_rest_screen_url( @hero )
    assert_response :success

    assert_select "a[href='#{hero_discard_corruption_card_screen_path(@hero)}']", false
  end

  test 'should show rest screen and show a link to discard corruption card' do
    @board.aasm_state = :hero_rest_screen
    @board.save!

    create( :isolated, board: @board, hero: @hero )
    @hero.favor = 2
    @hero.save!

    get hero_rest_screen_url( @hero )
    assert_response :success

    # puts response.body

    assert_select "a[href='#{hero_discard_corruption_card_screen_path(@hero)}']", true
  end

  test 'should rest and be redirected to movement screen' do
    @board.aasm_state = :hero_rest_screen
    @board.save!

    get hero_rest_rest_url( @hero )
    assert_redirected_to hero_movement_screen_url(@hero)
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


  test 'after movement, should create a combat' do
    @board.aasm_state = :hero_movement_screen
    @board.save!

    @board.create_monster( :agent, :old_forest, :monsters_pool_orange )

    assert_difference 'Combat.count' do
      post hero_move_url( @hero, params: { button: :old_forest, selected_cards: @hero.hand.first } )
    end

    assert @board.combat

    assert_redirected_to combat_setup_screen_board_combats_url(@board)
  end

  test 'after movement, if monster was nothing, should go to exploration screen' do
    @board.aasm_state = :hero_movement_screen
    @board.save!

    @board.create_monster( :nothing, :old_forest, :monsters_pool_orange )

    post hero_move_url( @hero, params: { button: :old_forest, selected_cards: @hero.hand.first } )
    assert_redirected_to hero_movement_screen_url(@hero)
    refute @board.monsters.where( location: :old_forest, code: :nothing ).exists?
  end

  test 'end game with heroes victory' do
    @board.aasm_state = 'hero_exploration'
    @board.story_marker_heroes = 16
    @board.save!

    @hero.turn = 2
    @hero.save!

    get next_step_hero_exploration_url( @hero )

    assert @board.reload.finished?
    assert_equal 'Heroes', @board.winner

    assert_redirected_to boards_url
    follow_redirect!

    assert_select 'td', 'Sauron'
    assert_select 'td', 'Argalad'
  end

  test 'end game with sauron victory' do
    @board.aasm_state = 'hero_exploration'
    @board.story_marker_conquest = 16
    @board.save!

    @hero.turn = 2
    @hero.save!

    create( :conquest_plot, board: @board )

    get next_step_hero_exploration_url( @hero )

    assert @board.reload.finished?
    assert_equal 'Sauron', @board.winner

    assert_redirected_to boards_url
    follow_redirect!

    assert_select 'td', 'Sauron'
    assert_select 'td', 'Argalad'
  end

  test 'end game with equality' do
    @board.aasm_state = 'hero_exploration'
    @board.story_marker_conquest = 16
    @board.story_marker_heroes = 16
    @board.save!

    @hero.turn = 2
    @hero.save!

    create( :conquest_plot, board: @board )

    get next_step_hero_exploration_url( @hero )

    assert @board.reload.finished?
    assert_equal 'Equality', @board.winner

    assert_redirected_to boards_url
    follow_redirect!

    assert_select 'td', 'Sauron'
    assert_select 'td', 'Argalad'
  end

  test 'should create the rinwraith and gothmog' do
    @board.aasm_state = 'hero_exploration'
    @board.story_marker_heroes = 6
    @board.save!

    @hero.turn = 2
    @hero.save!

    refute @board.mobs.where( code: :ringwraiths ).exists?
    refute @board.mobs.where( code: :gothmog ).exists?

    get next_step_hero_exploration_url( @hero )

    assert @board.mobs.where( code: :ringwraiths ).exists?
    assert @board.mobs.where( code: :gothmog ).exists?
  end

  test 'should create the witch-king' do
    @board.aasm_state = 'hero_exploration'
    @board.story_marker_heroes = 12
    @board.save!

    @hero.turn = 2
    @hero.save!

    refute @board.mobs.where( code: :witch_king ).exists?

    get next_step_hero_exploration_url( @hero )

    assert @board.mobs.where( code: :witch_king ).exists?
  end

end
