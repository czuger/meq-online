require 'test_helper'

class BoardsControllerTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board )

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new    $google_auth_hash
    get '/auth/google_oauth2'
    follow_redirect!
  end

  teardown do
    OmniAuth.config.test_mode = false
  end

  test 'should get index' do
    get boards_url( all: true )
    assert_response :success
  end

  test 'should get new' do
    get new_board_url
    assert_response :success
  end

  test 'should create board without sauron' do
    assert_difference('Board.count') do
      post boards_url, params: { max_heroes_count: 3, hero_1: 'eometh', hero_2: 'eleanor', hero_3: 'argalad' }
    end

    created_board_id = Board.pluck( :id ).max
    created_board = Board.find( created_board_id )

    assert_equal 3, created_board.current_heroes_count
    refute created_board.sauron_created

    assert_redirected_to boards_url
  end

  test 'should create board with sauron' do
    assert_difference('Board.count') do
      post boards_url, params: { sauron: true, max_heroes_count: 3, hero_1: 'eometh', hero_2: 'eleanor', hero_3: '' }
    end

    created_board_id = Board.pluck( :id ).max
    created_board = Board.find( created_board_id )

    assert_equal 2, created_board.current_heroes_count
    assert_equal 3, created_board.max_heroes_count
    assert created_board.sauron_created

    assert_redirected_to boards_url
  end

  test 'should create board with fewer heros' do
    assert_difference('Board.count') do
      post boards_url, params: { max_heroes_count: 2, hero_1: 'eometh', hero_2: '' }
    end

    created_board_id = Board.pluck( :id ).max
    created_board = Board.find( created_board_id )

    assert_equal 1, created_board.current_heroes_count
    assert_equal 2, created_board.max_heroes_count

    assert_equal 0, created_board.heroes.first.playing_order

    assert_redirected_to boards_url
  end

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

