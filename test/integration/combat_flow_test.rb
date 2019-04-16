require 'test_helper'

class CombatFlowTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )

    @sauron = create( :sauron, user: @user, board: @board, active: false )
    @hero = create( :hero, user: @user, board: @board, active: true, location: :the_shire )

    @mob = create( :monster, board: @board, location: :the_grey_havens )

    @board.sauron_created = true
    @board.current_heroes_count = 1
    @board.max_heroes_count = 1
    @board.users << @user

    @board.current_hero = @hero

    @board.aasm_state = 'movement'
    @board.save!

    create( :board_plot, board: @board )

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  test 'Test a full combat round' do
    post "/heros/#{@hero.id}/move", params: { selected_cards: '1', button: :the_grey_havens }
    assert_response :redirect
    follow_redirect!

    # puts @response.body
    # Should redirect to combat screen
    assert_select 'h3', 'Choose how to use your agility'

    assert @hero.reload.active
    refute @sauron.reload.active

    # # Simulate player disconection
    # We first have to find a better way to handle routing from status
    get '/boards'
    assert_response :success

    # puts @response.body

    assert_select 'td', 'Sauron'
    assert_select "a[href=?]", "/boards/#{@board.id}/combats/combat_setup_screen"

    get "/boards/#{@board.id}/combats/combat_setup_screen"
    assert_response :success
    assert_select 'h3', 'Choose how to use your agility'
  end

end
