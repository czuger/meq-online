require 'test_helper'

class BoardsControllerTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = Board.create_new_board

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:discord] = OmniAuth::AuthHash.new    $google_auth_hash
    post '/auth/discord'
    follow_redirect!
  end

  teardown do
    OmniAuth.config.test_mode = false
  end

  test 'should get index' do
    get boards_url( all: true )
    assert_response :success
  end

  test 'should get story screen' do
    get board_story_screen_url( @board )
    assert_response :success
  end

  test 'should get new' do
    get new_board_url
    assert_response :success
  end

  test 'should create board without sauron' do
    assert_difference('Board.count') do
      post boards_url, params: {}
    end

    created_board_id = Board.pluck( :id ).max
    created_board = Board.find( created_board_id )

    assert_equal 4, created_board.current_heroes_count
    assert created_board.sauron_created
    assert created_board.favors.count >= 1

    assert_redirected_to boards_url
  end

  # test 'should create board without sauron' do
  #   assert_difference('Board.count') do
  #     # post boards_url, params: { max_heroes_count: 3, hero_1: 'eometh', hero_2: 'eleanor', hero_3: 'argalad' }
  #     post boards_url, params: { max_heroes_count: 3, hero_1: 'eometh', hero_2: 'argalad' }
  #   end
  #
  #   created_board_id = Board.pluck( :id ).max
  #   created_board = Board.find( created_board_id )
  #
  #   assert_equal 2, created_board.current_heroes_count
  #   refute created_board.sauron_created
  #
  #   assert_redirected_to boards_url
  # end
  #
  # test 'should create board with sauron' do
  #   assert_difference('Board.count') do
  #     assert_difference('BoardPlot.count') do
  #       post boards_url, params: { sauron: true, max_heroes_count: 3, hero_1: 'eometh', hero_2: 'argalad', hero_3: '' }
  #     end
  #   end
  #
  #   created_board_id = Board.pluck( :id ).max
  #   created_board = Board.find( created_board_id )
  #
  #   assert_equal 2, created_board.current_heroes_count
  #   assert_equal 3, created_board.max_heroes_count
  #   assert created_board.sauron_created
  #
  #   created_plot = created_board.current_plots.first
  #   assert_equal 1, created_plot.plot_position
  #   assert_includes [0, 1, 2], created_plot.plot_card
  #
  #   assert_redirected_to boards_url
  # end
  #
  # test 'should create board with fewer heros' do
  #   assert_difference('Board.count') do
  #     post boards_url, params: { max_heroes_count: 2, hero_1: 'eometh', hero_2: '' }
  #   end
  #
  #   created_board_id = Board.pluck( :id ).max
  #   created_board = Board.find( created_board_id )
  #
  #   assert_equal 1, created_board.current_heroes_count
  #   assert_equal 2, created_board.max_heroes_count
  #
  #   assert_equal 0, created_board.heroes.first.playing_order
  #
  #   assert_redirected_to boards_url
  # end

  # test 'should show board' do
  #   get board_url(@board)
  #   assert_response :success
  # end

  # test 'should get edit' do
  #   get edit_board_url(@board)
  #   assert_response :success
  # end

  # test 'should update board' do
  #   patch board_url(@board), params: { board: { heroes: @board.heroes } }
  #   assert_redirected_to board_url(@board)
  # end

  # test 'should destroy board' do
  #   assert_difference('Board.count', -1) do
  #     delete board_url(@board)
  #   end
  #
  #   assert_redirected_to boards_url
  # end
end

