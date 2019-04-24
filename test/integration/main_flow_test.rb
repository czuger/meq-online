require 'test_helper'

class MainFlowTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board, favors: [ :the_shire, :the_grey_havens ] )

    @sauron = create( :sauron, user: @user, board: @board, active: true )
    @hero = create( :hero, user: @user, board: @board, active: false )

    @board.sauron_created = true
    @board.current_heroes_count = 1
    @board.max_heroes_count = 1
    @board.users << @user

    @board.aasm_state = 'sauron_setup_screen'
    @board.save!

    create( :board_plot, board: @board )
    create( :monster, board: @board )
    create( :minion, board: @board )

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  teardown do
    OmniAuth.config.test_mode = false
  end

  # This test only validate that the main flow is working
  test 'Must goes trough all steps and make a double turn then go to next turn' do

    get '/boards'
    assert_response :success
    # puts @response.body

    assert_select 'td', 'Argalad'
    assert_select "a[href=?]", "/sauron/#{@sauron.id}/setup_screen"

    refute @hero.reload.active
    assert @sauron.reload.active

    get "/sauron/#{@sauron.id}/setup_screen"
    assert_response :success

    get "/maps/#{@sauron.id}/edit"
    assert_response :success

    get "/sauron/#{@sauron.id}/plot_cards/draw_screen"
    assert_response :success

    get "/sauron/#{@sauron.id}/plot_cards/keep_screen"
    assert_response :success

    post "/sauron/#{@sauron.id}/plot_cards/keep", params: { selected_cards: '1, 2' }
    assert_response :redirect
    follow_redirect!

    get "/sauron/#{@sauron.id}/setup_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success

    refute @hero.reload.active
    assert @sauron.reload.active

    # We should be at sauron actions
    assert_select 'li', 'Select an action and validate it (this only place a marker on the map).'

    get "/sauron/#{@sauron.id}/sauron_actions/terminate"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    # We shoudl be back at board screen
    assert_select 'h1', 'Listing boards'

    # puts @response.body

    assert_select 'td', 'Sauron'
    assert_select "a[href=?]", "/heroes/#{@hero.id}/draw_cards_screen"

    assert @hero.reload.active
    refute @sauron.reload.active

    get "/heroes/#{@hero.id}/draw_cards_screen"
    assert_response :success

    assert @hero.reload.active
    refute @sauron.reload.active

    before_count = @hero.reload.hand.count

    post "/heroes/#{@hero.id}/draw_cards", params: { nb_cards: 5 }
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h3', 'Draw cards screen'

    assert_equal before_count + 5, @hero.reload.hand.count

    get "/heroes/#{@hero.id}/draw_cards_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h3', 'Rest screen'

    assert @hero.reload.active
    refute @sauron.reload.active

    get "/heroes/#{@hero.id}/rest_skip"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h3', 'Movement screen'

    assert @hero.reload.active
    refute @sauron.reload.active

    post "/heroes/#{@hero.id}/move", params: { selected_cards: '1', button: :the_grey_havens }
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h3', 'Exploration screen'

    assert @hero.reload.active
    refute @sauron.reload.active

    get "/heroes/#{@hero.id}/exploration_back_to_movement"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h3', 'Movement screen'

    post "/heroes/#{@hero.id}/move", params: { selected_cards: '2', button: :the_shire }
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h3', 'Exploration screen'

    assert @hero.reload.active
    refute @sauron.reload.active

    get "/heroes/#{@hero.id}/exploration_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h3', 'Draw cards screen'

    assert @hero.reload.active
    refute @sauron.reload.active

    get "/heroes/#{@hero.id}/draw_cards_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h3', 'Rest screen'

    assert @hero.reload.active
    refute @sauron.reload.active

    get "/heroes/#{@hero.id}/rest_skip"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h3', 'Movement screen'

    assert @hero.reload.active
    refute @sauron.reload.active

    post "/heroes/#{@hero.id}/move", params: { selected_cards: '3', button: :the_grey_havens }
    assert_response :redirect
    follow_redirect!
    assert_response :success
    assert_select 'h3', 'Exploration screen'

    assert @hero.reload.active
    refute @sauron.reload.active

    get "/heroes/#{@hero.id}/exploration_finished"
    assert_response :redirect
    follow_redirect!
    assert_response :success
    # We shoudl be back at board screen
    assert_select 'h1', 'Listing boards'

    assert_select 'td', 'Argalad'
    assert_select "a[href=?]", "/sauron/#{@sauron.id}/plot_cards/play_screen"

    refute @hero.reload.active
    assert @sauron.reload.active

    # puts @response.body
    # assert_select 'h1', 'Listing boards'
    #
    # assert_select 'td', 'Argalad'
    # assert_select "a[href=?]", "/plot_cards/#{@sauron.id}/play_screen"
    #
    # get "/plot_cards/#{@sauron.id}/play_screen"
    # assert_response :success
    # assert_select 'h3', 'Play plot card'
    #
    # get "/plot_cards/#{@sauron.id}/play_finished"
    # assert_response :redirect
    # follow_redirect!
    # assert_response :success
    # assert_select 'li', 'Select an action and validate it (this only place a marker on the map).'
  end
end
