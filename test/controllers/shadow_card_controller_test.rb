require 'test_helper'

class ShadowCardControllerTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )
    @board.users << @user
    @sauron = create( :sauron, board: @board, user: @user )

    @board.aasm_state = 'sauron_actions'
    @board.save!

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  test 'should get discard_screen' do
    get plot_cards_discard_screen_url @sauron
    assert_response :success
  end

  # test 'should get play_screen' do
  #   get play_screen_sauron_shadow_cards_url @sauron
  #   assert_response :success
  # end

  test 'should get play_screen at start of hero turn' do
    get start_hero_turn_play_screen_sauron_shadow_cards_url @sauron
    assert_response :success
  end

  test 'should play card' do
    @sauron.shadow_cards << 8
    @sauron.save!
    post play_sauron_shadow_cards_url @sauron, params: { selected_card: 8 }
    # assert_redirected_to sauron_url(@sauron)
    refute_includes  @sauron.reload.shadow_cards, 8
  end

  test "should fail because user don't have the card in hand card" do
    assert_raise do
      post shadow_cards_play_url @sauron, params: { selected_card: 8, card_slot: 'shadow-card-1' }
    end
  end

  test 'should get draw_screen' do
    get draw_screen_sauron_shadow_cards_url @sauron
    assert_response :success
  end

  test 'should draw cards' do
    @sauron.drawn_shadow_cards = []
    @sauron.save!
    post draw_sauron_shadow_cards_url @sauron, params: { nb_cards: 6 }
    # assert_redirected_to keep_screen_sauron_shadow_cards_url(@sauron)
    assert_equal 6, @sauron.reload.drawn_shadow_cards.count
  end

  test 'should keep cards' do
    @sauron.shadow_cards = []
    @sauron.drawn_shadow_cards = [ 7, 8, 9 ]
    @sauron.save!
    post keep_sauron_shadow_cards_url @sauron, params: { selected_cards: [ 8, 9 ].join(',' ) }
    assert_redirected_to edit_sauron_action_url(@sauron)
    assert_empty  @sauron.reload.drawn_shadow_cards
    assert_equal [ 8, 9 ], @sauron.reload.shadow_cards
  end

end
