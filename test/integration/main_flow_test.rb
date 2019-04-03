require 'test_helper'

class MainFlowTest < ActionDispatch::IntegrationTest

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

    @board.aasm_state = 'sauron_setup'
    @board.save!

    create( :board_plot, board: @board )

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  teardown do
    OmniAuth.config.test_mode = false
  end

  # This test only validate that the main flow is working
  test 'Must goes trough all steps and make a full turn' do

    get '/boards'
    assert_response :success
    # puts @response.body

    assert_select 'td', 'Argalad'
    assert_select "a[href=?]", "/sauron/#{@sauron.id}/setup"

    get "/sauron/#{@sauron.id}/setup"
    assert_response :success

    get "/sauron/#{@sauron.id}/setup_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success

    # We should be at sauron actions
    assert_select 'li', 'Select an action and validate it (this only place a marker on the map).'

    get "/sauron_actions/#{@sauron.id}/terminate"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    # We shoudl be back at board screen
    assert_select 'h1', 'Listing boards'

    assert_select 'td', 'Sauron'
    assert_select "a[href=?]", "/heros/#{@hero.id}/draw_cards_screen"

    get "/heros/#{@hero.id}/draw_cards_screen"
    assert_response :success

    get "/heros/#{@hero.id}/draw_cards_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h1', 'Listing boards'

    assert_select 'td', 'Argalad'
    assert_select "a[href=?]", "/sauron/#{@sauron.id}/shadow_cards/start_hero_turn_play_card_screen"

    get "/sauron/#{@sauron.id}/shadow_cards/start_hero_turn_play_card_screen"
    assert_response :success
    assert_select 'h3', 'Select shadow card to play'

    get "/sauron/#{@sauron.id}/shadow_cards/start_hero_turn_play_card_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h1', 'Listing boards'

    assert_select 'td', 'Sauron'
    assert_select "a[href=?]", "/heros/#{@hero.id}/rest_screen"

    get "/heros/#{@hero.id}/rest_screen"
    assert_response :success

    get "/heros/#{@hero.id}/rest_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h1', 'Movement preparation'

    get "/heros/#{@hero.id}/movement_preparation_steps/terminate"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h1', 'Listing boards'

    assert_select 'td', 'Argalad'
    assert_select "a[href=?]", "/sauron/#{@sauron.id}/movement_break_schedule_screen"

    get "/sauron/#{@sauron.id}/movement_break_schedule_screen"
    assert_response :success

    get "/sauron/#{@sauron.id}/movement_break_schedule_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h1', 'Listing boards'

    assert_select 'td', 'Sauron'
    assert_select "a[href=?]", "/heros/#{@hero.id}/movement_screen"

    get "/heros/#{@hero.id}/movement_screen"
    assert_response :success

    get "/heros/#{@hero.id}/movement_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h3', 'Exploration screen'

    get "/heros/#{@hero.id}/exploration_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h3', 'Encounter screen'

    get "/heros/#{@hero.id}/encounter_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h1', 'Listing boards'

    assert_select 'td', 'Argalad'
    assert_select "a[href=?]", "/sauron/#{@sauron.id}/story_screen"

    get "/sauron/#{@sauron.id}/story_screen"
    assert_response :success

    get "/sauron/#{@sauron.id}/story_step_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    # assert_select 'h3', 'Encounter screen'

    get "/sauron/#{@sauron.id}/event/edit"
    assert_response :success
  end

  # This test only validate that the main flow is working
  test 'Test user switch at the end of user turn' do

    @thalin = create( :hero, name_code: :thalin, name: 'Thalin', user: @user, board: @board, active: false )
    @thalin.save!

    @argalad = @hero

    @board.sauron_created = true
    @board.current_heroes_count = 2
    @board.max_heroes_count = 2

    @board.current_hero = @argalad

    @board.aasm_state = 'encounter'
    @board.save!

    get "/heros/#{@hero.id}/encounter_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h1', 'Listing boards'

    # puts @response.body

    assert_select 'td', 'Sauron'
    assert_select 'td', 'Thalin'
    assert_select "a[href=?]", "/sauron/#{@sauron.id}/shadow_cards/start_hero_turn_play_card_screen"
  end

end
