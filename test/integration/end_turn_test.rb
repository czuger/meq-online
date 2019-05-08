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

    @board.aasm_state = 'sauron_setup_screen'
    @board.save!

    create( :board_plot, board: @board )

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  test 'Test 1 player switch to second turn.' do
    @board.aasm_state = 'hero_exploration'
    @board.save!

    get next_step_hero_exploration_url(@hero)
    assert_response :redirect
    follow_redirect!
    assert_response :success

    assert_equal 2, @hero.reload.turn
    assert_select 'b', 'Hand:'

    get '/boards'
    assert_select 'td', 'Sauron'
    assert_select 'a[href=?]', hero_draw_cards_screen_path(@hero)

    get "#{hero_draw_cards_screen_path(@hero)}"
    assert_response :success
    # puts @response.body
    assert_select 'h3', 'Draw cards screen'
    assert_select 'a[href=?]', "/heroes/#{@hero.id}/draw_cards_finished"

    get "/heroes/#{@hero.id}/draw_cards_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    # puts @response.body
    # assert_select 'h3', 'Rest screen'
  end

  test 'Test 1 player switch to sauron after finishing his second turn.' do
    @board.aasm_state = 'hero_exploration'
    @board.save!

    @hero.turn = 2
    @hero.save!

    get next_step_hero_exploration_url(@hero)
    assert_response :redirect
    follow_redirect!
    assert_response :success

    assert_equal 2, @board.reload.turn
    assert_equal 2, @board.story_marker_heroes
    assert_equal 3, @board.story_marker_corruption

    # puts @response.body

    assert_select 'td', 'Argalad'
    assert_select "a[href=?]", "/sauron/#{@sauron.id}/plot_cards/play_screen"
  end

  test 'Test 2 player switch to next player.' do

    @thalin = create( :hero, name_code: :thalin, name: 'Thalin', user: @user, board: @board, active: false )
    @thalin.save!

    @argalad = @hero

    @board.sauron_created = true
    @board.current_heroes_count = 2
    @board.max_heroes_count = 2

    @board.current_hero = @argalad

    @board.aasm_state = 'hero_exploration'
    @board.save!

    get next_step_hero_exploration_url(@hero)
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h1', 'Listing boards'

    # puts @response.body

    assert_select 'td', 'Argalad'
    assert_select 'td', 'Sauron'
    assert_select "a[href=?]", hero_draw_cards_screen_path(@thalin)
  end

  test 'Test 2 player switch to sauron (after everybody has played).' do

    @thalin = create( :hero, name_code: :thalin, name: 'Thalin', user: @user, board: @board, active: false, turn_finished: true )
    @thalin.save!

    @argalad = @hero
    @argalad.turn_finished = true
    @argalad.save!

    @board.sauron_created = true
    @board.current_heroes_count = 2
    @board.max_heroes_count = 2

    @board.current_hero = @argalad

    @board.aasm_state = 'hero_exploration'
    @board.save!

    get next_step_hero_exploration_url(@hero)
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h1', 'Listing boards'

    # puts @response.body

    assert_select 'td', 'Argalad'
    assert_select 'td', 'Thalin'
    assert_select "a[href=?]", "/sauron/#{@sauron.id}/plot_cards/play_screen"
  end

end
