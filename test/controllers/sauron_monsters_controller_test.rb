require 'test_helper'

class SauronMobsControllerTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )
    @hero = create( :hero, user: @user, board: @board )
    @sauron = create( :sauron, user: @user, board: @board )
    @board_message = create( :board_message, sender: @sauron, reciever: @hero )
    @board.users << @user

    @board.create_monster( :agent, :old_forest, :mobs_pool_orange )

    @black_serpent = create( :black_serpent, board: @board, life: 9 )

    @board.aasm_state = 'edit_sauron_sauron_actions'
    @board.save!

    GameData::LocationsMonsters.new.fill_board(@board)

    connection_for_tests
  end

  test 'should get index' do
    get sauron_sauron_mobs_url(@sauron)
    assert_response :success
  end

  test 'should get new' do
    get new_sauron_sauron_mob_url(@sauron)
    assert_response :success
  end

  test 'should get edit' do
    get edit_sauron_sauron_mob_url(@sauron, @black_serpent)
    assert_response :success
  end

  test 'should move_mob' do
    patch sauron_sauron_mob_url(@sauron, @black_serpent, params: {button: :fornost } )
    assert_redirected_to sauron_sauron_mobs_url(@sauron)
  end

  test 'should create a mob' do
    post sauron_sauron_mobs_url(@sauron, params: { location: :fornost } )
    assert_redirected_to sauron_sauron_mobs_url(@sauron)
  end

  test 'should heal a mob' do
    get sauron_sauron_mob_heal_url(@sauron, @black_serpent)
    assert_equal 11, @black_serpent.reload.life
    assert_redirected_to sauron_sauron_mobs_url(@sauron)
  end

end
