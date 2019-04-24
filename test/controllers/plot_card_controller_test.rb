require 'test_helper'

class PlotCardControllerTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )
    @board.users << @user
    @sauron = create( :sauron, board: @board, user: @user )

    @board.aasm_state = 'edit_sauron_sauron_actions'
    @board.save!

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  test 'should get discard_screen' do
    get discard_screen_sauron_plot_cards_url @sauron
    assert_response :success
  end

  test 'should discard empty slot' do
    post discard_sauron_plot_cards_url @sauron, params: { selected_card: 1 }
    assert_redirected_to play_screen_sauron_plot_cards_url(@sauron)
  end

  test 'should discard non empty slot' do
    @board.current_plots.create!( plot_position: 1, plot_card: 8, affected_location: 'dummy',
        story_type: 'dummy', story_advance: 1, favor_to_discard: 3 )
    @board.save!
    post discard_sauron_plot_cards_url @sauron, params: { selected_card: 1 }
    assert_redirected_to play_screen_sauron_plot_cards_url(@sauron)
    assert_empty @board.reload.current_plots
  end

  test 'should get play_screen' do
    get play_screen_sauron_plot_cards_url @sauron
    assert_response :success
  end

  test 'should play card' do
    @board.aasm_state = 'play_screen_sauron_plot_cards'
    @board.save!

    @sauron.plot_cards << 8
    @sauron.save!
    post play_sauron_plot_cards_url @sauron, params: { selected_card: 8, card_slot: 2 }
    assert_redirected_to edit_sauron_sauron_actions_url(@sauron)
    assert_equal 8, @board.get_plot_card(2)
    refute_includes  @sauron.reload.plot_cards, 8
  end

  test "should fail because user don't have the card in hand card" do
    assert_raise do
      post play_sauron_plot_cards_url @sauron, params: { selected_card: 8, card_slot: 'plot-card-1' }
    end
  end

  test 'should fail because slot is already used' do
    @board.current_plots.create!( plot_position: 1, plot_card: 8, affected_location: 'dummy',
                                  story_type: 'dummy', story_advance: 1, favor_to_discard: 3  )
    assert_raise do
      post play_sauron_plot_cards_url @sauron, params: { selected_card: 8, card_slot: 1 }
    end
  end

  test 'should get draw_screen' do
    get draw_screen_sauron_plot_cards_url @sauron
    assert_response :success
  end

  test 'should draw cards' do
    @sauron.drawn_plot_cards = []
    @sauron.save!
    post draw_sauron_plot_cards_url @sauron, params: { nb_cards: 6 }
    assert_redirected_to keep_screen_sauron_plot_cards_url(@sauron)
    assert_equal 6, @sauron.reload.drawn_plot_cards.count
  end

  # test 'should keep cards' do
  #   @sauron.plot_cards = []
  #   @sauron.drawn_plot_cards = [ 7, 8, 9 ]
  #   @sauron.save!
  #   post plot_cards_keep_url @sauron, params: { selected_cards: [ 8, 9 ].join(',' ) }
  #   assert_redirected_to play_screen_sauron_plot_cards_url(@sauron)
  #   assert_empty  @sauron.reload.drawn_plot_cards
  #   assert_equal [ 8, 9 ], @sauron.reload.plot_cards
  # end

end
