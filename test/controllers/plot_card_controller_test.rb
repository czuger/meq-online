require 'test_helper'

class PlotCardControllerTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )
    @board.users << @user
    @sauron = create( :sauron, board: @board, user: @user )

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  test 'should get discard_screen' do
    get plot_cards_discard_screen_url @sauron
    assert_response :success
  end

  test 'should discard empty slot' do
    post plot_cards_discard_url @sauron, params: { selected_card: 'plot-card-1' }
    assert_redirected_to plot_cards_discard_screen_url(@sauron)
  end

  test 'should discard non empty slot' do
    @board.current_plots['plot-card-1'] = 8
    @board.save!
    post plot_cards_discard_url @sauron, params: { selected_card: 'plot-card-1' }
    assert_redirected_to plot_cards_discard_screen_url(@sauron)
    assert_empty @board.reload.current_plots
  end

  test 'should get play_screen' do
    get plot_cards_play_screen_url @sauron
    assert_response :success
  end

  test 'should play card' do
    @sauron.plot_cards << 8
    @sauron.save!
    post plot_cards_play_url @sauron, params: { selected_card: 8, card_slot: 'plot-card-2' }
    assert_redirected_to plot_cards_play_screen_url(@sauron)
    assert_equal '8', @board.reload.current_plots['plot-card-2']
    refute_includes  @sauron.reload.plot_cards, 8
  end

  test "should fail because user don't have the card in hand card" do
    assert_raise do
      post plot_cards_play_url @sauron, params: { selected_card: 8, card_slot: 'plot-card-1' }
    end
  end

  test 'should fail because slot is already used' do
    @board.current_plots['plot-card-1'] = 10
    @board.save!
    assert_raise do
      post plot_cards_play_url @sauron, params: { selected_card: 8, card_slot: 'plot-card-1' }
    end
  end

  test 'should get draw_screen' do
    get plot_cards_draw_screen_url @sauron
    assert_response :success
  end

  test 'should draw cards' do
    @sauron.drawn_plot_cards = []
    @sauron.save!
    post plot_cards_draw_url @sauron, params: { nb_cards: 6 }
    assert_redirected_to plot_cards_draw_screen_url(@sauron)
    assert_equal 6, @sauron.reload.drawn_plot_cards.count
  end

end
