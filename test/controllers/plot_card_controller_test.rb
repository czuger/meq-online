require 'test_helper'

class PlotCardControllerTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @plot_cards = GameData::Plots.new

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
    @sauron.plot_cards = @plot_cards.all_cards
    @sauron.save!

    get play_screen_sauron_plot_cards_url @sauron

    assert_equal [ 3, 4, 7, 9, 13, 16 ],assert_select("img[class='medium-card']").map{ |e| e.attribute('card_id').value.to_i }.sort
    assert_response :success
  end

  test 'should get play_screen and a dark messenger should be selectable' do
    @sauron.plot_cards = @plot_cards.all_cards
    @sauron.save!

    @board.create_monster( :mouth_of_sauron, :erebor )

    get play_screen_sauron_plot_cards_url @sauron

    assert_equal [ 4, 7, 9, 13, 16 ],assert_select("img[class='medium-card']").map{ |e| e.attribute('card_id').value.to_i }.sort
    assert_response :success
  end

  test 'should get play_screen and osgilliath invaded should be selectable' do
    @sauron.plot_cards = @plot_cards.all_cards
    @sauron.save!

    @board.create_monster( :orc, :osgiliath, pool_key: :dummy )

    get play_screen_sauron_plot_cards_url @sauron

    assert_equal [ 3, 7, 9, 13, 16 ],assert_select("img[class='medium-card']").map{ |e| e.attribute('card_id').value.to_i }.sort
    assert_response :success
  end

  test 'should get play_screen and wormtongue taints théoden should be selectable' do
    @sauron.plot_cards = @plot_cards.all_cards
    @sauron.save!

    create( :saruman_falls_to_corruption, board: @board )

    get play_screen_sauron_plot_cards_url @sauron

    assert_equal [ 3, 4, 9, 13, 16 ],assert_select("img[class='medium-card']").map{ |e| e.attribute('card_id').value.to_i }.sort
    assert_response :success
  end

  test 'should get play_screen and orcs in the mountains should be selectable' do
    @sauron.plot_cards = @plot_cards.all_cards
    @sauron.save!

    @board.influence['mount_gundabad'] = 3
    @board.influence['the_ruins_of_angmar'] = 3
    @board.influence['the_high_pass'] = 2
    @board.save!

    get play_screen_sauron_plot_cards_url @sauron

    assert_equal [ 3, 4, 7, 13, 16 ],assert_select("img[class='medium-card']").map{ |e| e.attribute('card_id').value.to_i }.sort
    assert_response :success
  end

  test 'should get play_screen and smeagol escapes should be selectable' do
    @sauron.plot_cards = @plot_cards.all_cards
    @sauron.save!

    @board.create_monster( :mouth_of_sauron, :the_forest_trail )

    get play_screen_sauron_plot_cards_url @sauron

    assert_equal [ 3, 4, 7, 9, 16 ],assert_select("img[class='medium-card']").map{ |e| e.attribute('card_id').value.to_i }.sort
    assert_response :success
  end

  test 'should get play_screen and raise of the uruk-hai should be selectable' do
    @sauron.plot_cards = @plot_cards.all_cards
    @sauron.save!

    @board.influence['isengard'] = 3
    @board.save!

    get play_screen_sauron_plot_cards_url @sauron

    assert_equal [ 3, 4, 7, 9, 13 ],assert_select("img[class='medium-card']").map{ |e| e.attribute('card_id').value.to_i }.sort
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

  test 'should play card - in case of smeagol card, should lead to gollum card choice' do
    @board.aasm_state = 'play_screen_sauron_plot_cards'
    @board.save!

    @sauron.plot_cards << 13
    @sauron.save!

    post play_sauron_plot_cards_url @sauron, params: { selected_card: 13, card_slot: 2 }
    assert_equal 13, @board.get_plot_card(2)
    refute_includes  @sauron.reload.plot_cards, 13

    assert_redirected_to look_for_gollum_cards_sauron_plot_cards_url(@sauron)
    follow_redirect!
  end

  test 'should play card - in case of gollum card, should remove other gollum cards' do
    @board.aasm_state = 'play_screen_sauron_plot_cards'
    @board.save!

    create( :gollum_is_captured, board: @board )

    @sauron.plot_cards << 12
    @sauron.save!

    post play_sauron_plot_cards_url @sauron, params: { selected_card: 12, card_slot: 2 }
    assert_equal 12, @board.get_plot_card(2)
    refute_includes  @sauron.reload.plot_cards, 12

    refute_equal 5, @board.reload.get_plot_card(1)
    assert_includes  @board.reload.plot_discard, 5

    assert_redirected_to edit_sauron_sauron_actions_url(@sauron)
  end


  test 'get one of the proposed gollum card' do
    @board.aasm_state = 'play_screen_sauron_plot_cards'
    @board.save!

    post get_gollum_card_sauron_plot_cards_url @sauron, params: { selected_card: 12 }
    assert_includes @sauron.reload.plot_cards, 12
    assert_redirected_to edit_sauron_sauron_actions_url(@sauron)
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
