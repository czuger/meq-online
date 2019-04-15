require 'test_helper'

class SauronMonstersControllerTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )
    @hero = create( :hero, user: @user, board: @board )
    @sauron = create( :sauron, user: @user, board: @board )
    @board_message = create( :board_message, sender: @sauron, reciever: @hero )
    @board.users << @user

    @board.create_monster( :agent, :old_forest, :monsters_pool_orange )

    @black_serpent = create( :minion, board: @board )

    @board.aasm_state = 'sauron_actions'
    @board.save!

    GameData::LocationsMonsters.new.fill_board(@board)

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  test 'should get index' do
    get sauron_sauron_monsters_url(@sauron)
    assert_response :success
  end

  test 'should get new' do
    get new_sauron_sauron_monster_url(@sauron)
    assert_response :success
  end

  test 'should get edit' do
    get edit_sauron_sauron_monster_url(@sauron, @black_serpent)
    assert_response :success
  end

  test 'should move_monster' do
    patch sauron_sauron_monster_url(@sauron, @black_serpent, params: { button: :fornost } )
    assert_redirected_to sauron_sauron_monsters_url(@sauron)
  end

  test 'should create a monster' do
    post sauron_sauron_monsters_url(@sauron, params: { location: :fornost } )
    assert_redirected_to sauron_sauron_monsters_url(@sauron)
  end

end
