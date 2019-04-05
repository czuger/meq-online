require 'test_helper'

class EndTurnTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )

    @sauron = create( :sauron, user: @user, board: @board, active: true )
    @hero = create( :hero, user: @user, board: @board, active: false )

    @board.sauron_created = true
    @board.current_heroes_count = 1
    @board.max_heroes_count = 1
    @board.users << @user

    @board.current_hero = @hero

    @board.aasm_state = 'sauron_setup'
    @board.save!

    create( :board_plot, board: @board )

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  test 'Test 1 player switch to second turn.' do
    @board.aasm_state = 'exploration'
    @board.save!

    get "/heros/#{@hero.id}/exploration_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success

    # puts @response.body

    assert_select 'b', 'Hand:'
  end

  test 'Test 1 player switch to sauron after finishing his second turn.' do
    @board.aasm_state = 'exploration'
    @board.save!

    @hero.turn = 2
    @hero.save!

    get "/heros/#{@hero.id}/exploration_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success

    # puts @response.body

    assert_select 'td', 'Argalad'
    assert_select "a[href=?]", "/plot_cards/#{@sauron.id}/play_screen"
  end

  test 'Test user switch at the end of user turn (2 players)' do

    @thalin = create( :hero, name_code: :thalin, name: 'Thalin', user: @user, board: @board, active: false )
    @thalin.save!

    @argalad = @hero

    @board.sauron_created = true
    @board.current_heroes_count = 2
    @board.max_heroes_count = 2

    @board.current_hero = @argalad

    @board.aasm_state = 'exploration'
    @board.save!

    get "/heros/#{@hero.id}/exploration_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h1', 'Listing boards'

    # puts @response.body

    assert_select 'td', 'Argalad'
    assert_select 'td', 'Sauron'
    assert_select "a[href=?]", "/heros/#{@thalin.id}/rest_screen"
  end

end
