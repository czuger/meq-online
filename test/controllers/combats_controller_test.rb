require 'test_helper'

class CombatsControllerTest < ActionDispatch::IntegrationTest
  setup do
    OmniAuth.config.test_mode = true

    @game_data_heroes ||= GameData::Heroes.new
    @game_data_mobs_cards ||= GameData::MobsCards.new

    @user = create( :user )
    @board = create( :board )

    @hero = create( :hero, user: @user, board: @board )
    @sauron = create( :sauron, user: @user, board: @board )

    @mob = create( :orc, board: @board )
    @combat = @board.create_combat( @hero, @mob )

    @board.combat.temporary_hero_strength = 5
    @board.combat.save!

    @board.users << @user

    @board.aasm_state = 'play_combat_card_screen_board_combats'
    @board.current_hero = @hero
    @board.save!

    @hero_aimed_shot = @game_data_heroes.get_card_number_by_name( :argalad, 'Aimed Shot' )
    @mob_aimed_shot = @game_data_mobs_cards.get_card_number_by_name( 'zealot', 'Aimed Shot' )

    connection_for_tests
  end

  teardown do
    OmniAuth.config.test_mode = false
  end

  test 'should show combat with exhausted cards' do
    create( :argalad_quick_shot, combat: @board.combat )
    create( :hero_exhausted, combat: @board.combat )

    create( :behemoth_precision, combat: @board.combat )
    create( :mob_exhausted, combat: @board.combat )

    get board_combats_url(@board)
    assert_response :success
  end

  test 'should get hero_setup_new' do
    get combat_setup_screen_board_combats_url(@board)
    assert_response :success
  end

  test 'should start combat with strength increase' do
    @board.aasm_state = 'combat_setup_screen_board_combats'
    @board.save!

    post combat_setup_board_combats_url(@board, button: :increase)

    assert_equal 8, @board.combat.reload.temporary_hero_strength

    assert_redirected_to play_combat_card_screen_board_combats_url(@board, @hero)
  end

  test 'should start combat with strength increase but hero should loose a random card because of the cowardly corruption' do
    @board.aasm_state = 'combat_setup_screen_board_combats'
    @board.save!

    @hero.hand = [ 1, 2, 3 ]
    @hero.rest_pool = []
    @hero.save!

    create( :cowardly, board: @board, hero: @hero )

    post combat_setup_board_combats_url(@board, button: :increase)

    assert_equal 2, @hero.reload.hand.count
    assert_equal 1, @hero.rest_pool.count

    assert_redirected_to play_combat_card_screen_board_combats_url(@board, @hero)

    get board_logs_url(@board)
    assert_select 'td', true, "Has lost a random hero card because of the cowardly corruption."
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

    post play_combat_card_hero_board_combats_url(@board, selected_card: @hero.hand.sort.first)
    assert_redirected_to boards_url

    assert_difference 'CombatCardPlayed.count', 2 do
      post play_combat_card_mob_board_combats_url(@board, selected_card: @mob.hand.sort.first)
    end
    assert_redirected_to board_combats_path(@board)

    # follow_redirect!
  end

  test 'On mob and hero exhaustion, should switch to hero next turn' do
    @hero.hand << @hero_aimed_shot
    @hero.save!
    @mob.hand << @mob_aimed_shot
    @mob.save!

    @board.combat.hero_strength_used = 4
    @board.combat.mob_strength_used = 3
    @board.combat.save!

    post play_combat_card_hero_board_combats_url(@board, selected_card: @hero_aimed_shot)
    assert_redirected_to boards_url

    assert_difference 'CombatCardPlayed.count', 2 do
      post play_combat_card_mob_board_combats_url(@board, selected_card: @mob_aimed_shot)
    end
    assert_redirected_to board_combats_url(@board)
    follow_redirect!

    assert_select 'h3', 'Hero and mob where both exhausted.'

    get terminate_board_combats_url( @board )
    assert_redirected_to hero_draw_cards_screen_url(@hero)

    assert @hero.reload.active
    refute @sauron.reload.active

    refute Mob.where( code: @mob.code, location: @mob.location ).exists?
  end

  test 'On mob and hero exhaustion, should switch to sauron if it was the second hero turn' do
    @hero.hand << @hero_aimed_shot
    @hero.turn = 2
    @hero.save!
    @mob.hand << @mob_aimed_shot
    @mob.save!

    @board.combat.hero_strength_used = 4
    @board.combat.mob_strength_used = 3
    @board.combat.save!

    post play_combat_card_hero_board_combats_url(@board, selected_card: @hero_aimed_shot)
    assert_redirected_to boards_url

    assert_difference 'CombatCardPlayed.count', 2 do
      post play_combat_card_mob_board_combats_url(@board, selected_card: @mob_aimed_shot)
    end
    assert_redirected_to board_combats_url(@board)
    follow_redirect!

    assert_select 'h3', 'Hero and mob where both exhausted.'

    get terminate_board_combats_url( @board )
    assert_redirected_to boards_url

    refute @hero.reload.active
    assert @sauron.reload.active

    refute Mob.where( code: @mob.code, location: @mob.location ).exists?
  end

  test 'On mob and hero exhaustion, should switch to next hero if we have more than one player' do
    @hero.hand << @hero_aimed_shot
    @hero.save!
    @mob.hand << @mob_aimed_shot
    @mob.save!

    @second_hero = create( :hero, user: @user, board: @board, name_code: :foo )

    @board.combat.hero_strength_used = 4
    @board.combat.mob_strength_used = 3
    @board.combat.save!

    @board.current_heroes_count = 2
    @board.save!

    post play_combat_card_hero_board_combats_url(@board, selected_card: @hero_aimed_shot)
    assert_redirected_to boards_url

    assert_difference 'CombatCardPlayed.count', 2 do
      post play_combat_card_mob_board_combats_url(@board, selected_card: @mob_aimed_shot)
    end
    assert_redirected_to board_combats_url(@board)
    follow_redirect!

    assert_select 'h3', 'Hero and mob where both exhausted.'

    get terminate_board_combats_url( @board )
    assert_redirected_to boards_url

    refute @hero.reload.active
    refute @sauron.reload.active

    assert @second_hero.reload.active

    refute Mob.where( code: @mob.code, location: @mob.location ).exists?
  end

  # TODO : add a test with a hero linked to another user.
  test 'On hero defeat, should switch to hero next turn' do
    @hero.hand = [ @hero_aimed_shot ]
    @hero.life_pool = [ 1 ]
    @hero.save!

    @mob.hand << @mob_aimed_shot
    @mob.save!

    post play_combat_card_hero_board_combats_url(@board, selected_card: @hero_aimed_shot)
    assert_redirected_to boards_url

    assert_difference 'CombatCardPlayed.count', 2 do
      post play_combat_card_mob_board_combats_url(@board, selected_card: @mob_aimed_shot)
    end
    assert_redirected_to board_combats_url(@board)
    follow_redirect!

    assert_select 'h3', 'Hero was defeated !!!'

    get terminate_board_combats_url( @board )
    assert_redirected_to hero_draw_cards_screen_url(@hero)

    assert @hero.reload.active
    refute @sauron.reload.active

    assert_equal 'fornost', @hero.reload.location

    refute Mob.where( code: @mob.code, location: @mob.location ).exists?
  end

  test 'should show cards loss screen' do
    get cards_loss_screen_board_combats_url(@board)
    assert_response :success

    # puts @response.body
    # puts assert_select 'img'

    assert_select "img[class='small-card selectable-card-selection-multiple']"
  end

  test 'should loose selected cards' do
    @hero.life_pool = []
    @hero.temporary_damages = 3
    @hero.save!

    @board.aasm_state = 'cards_loss_screen_board_combats'
    @board.save!

    assert_difference '@hero.reload.hand.count', -3 do
      assert_difference '@hero.reload.damage_pool.count', 3 do
        post cards_loss_board_combats_path(@board), params: { selected_cards: @hero.hand.sort.first(3).join(',') }
      end
    end

    assert_redirected_to board_combats_url(@board)
  end

  test 'On minion defeat, minion should be removed' do
    @hero.hand << @hero_aimed_shot
    @hero.save!

    @mob = create( :mouth_of_sauron, board: @board, life: 1 )

    @combat.destroy!

    @combat = @board.create_combat( @hero, @mob )
    @combat.temporary_hero_strength = 50
    @combat.save!

    @mob.hand << @mob_aimed_shot
    @mob.save!

    post play_combat_card_hero_board_combats_url(@board, selected_card: @hero_aimed_shot)
    post play_combat_card_mob_board_combats_url(@board, selected_card: @mob_aimed_shot)

    get terminate_board_combats_url( @board )
    assert_redirected_to hero_movement_screen_url( @hero )

    refute @board.minions.where( code: :mouth_of_sauron ).exists?
  end

  test 'Undefeated minions should not be removed' do
    @hero.hand << @hero_aimed_shot
    @hero.save!

    @mob = create( :mouth_of_sauron, board: @board, life: 50 )

    @combat.destroy!

    @combat = @board.create_combat( @hero, @mob )
    @combat.temporary_hero_strength = 1
    @combat.save!

    @mob.hand << @mob_aimed_shot
    @mob.strength = 1
    @mob.save!

    post play_combat_card_hero_board_combats_url(@board, selected_card: @hero_aimed_shot)
    post play_combat_card_mob_board_combats_url(@board, selected_card: @mob_aimed_shot)

    get terminate_board_combats_url( @board )
    assert @mob.reload
  end

  test 'On ringwraith defeat, they should not be defeated, instead : moved to minas tirith' do
    @hero.hand << @hero_aimed_shot
    @hero.save!

    @mob = @board.create_monster( :ringwraiths, :the_shire )

    @combat.destroy!

    @combat = @board.create_combat( @hero, @mob )
    @combat.temporary_hero_strength = 50
    @combat.save!

    @mob.hand << 0
    @mob.life = 1
    @mob.save!

    post play_combat_card_hero_board_combats_url(@board, selected_card: @hero_aimed_shot)
    post play_combat_card_mob_board_combats_url(@board, selected_card: 0)

    get terminate_board_combats_url( @board )
    assert_redirected_to hero_movement_screen_url( @hero )

    assert @mob.reload
    assert @board.minions.where( code: :ringwraiths ).exists?
    assert_equal 'minas_tirith', @mob.location
  end

end
