require 'test_helper'

class ExplorationsControllerTest < ActionDispatch::IntegrationTest

  setup do
    OmniAuth.config.test_mode = true

    @user = create( :user )
    @board = create( :board, favors: [ :old_forest, :bree ] )
    @hero = create( :hero, user: @user, board: @board, location: :bree )
    @sauron = create( :sauron, user: @user, board: @board )
    @board.users << @user

    @board.current_hero = @hero
    @board.save!

    $google_auth_hash[:uid] = @user.uid
    OmniAuth.config.mock_auth[:google_oauth2] = OmniAuth::AuthHash.new $google_auth_hash
    post '/auth/google_oauth2'
    follow_redirect!

    # pp User.all
  end

  test 'should not be able to see gandalf, because hero is isolated' do
    create( :isolated, board: @board, hero: @hero )

    @board.favors = []
    @board.characters[:gandalf] = :bree
    @board.save!
    get hero_exploration_url(@hero)

    # puts @response.body
    assert_select 'input[class=form-check-input]', false

    get board_logs_url(@board)
    # puts @response.body
    assert_select 'td', true, "Was't able to talk with Gandalf because of the isolated corruption."
  end

  test 'should get a favor' do
    put hero_exploration_url( @hero, tokens: { favor: [ :favor ] } )
    assert_redirected_to hero_exploration_url(@hero)
    assert_equal 1, @hero.reload.favor
  end

  test 'should get only one favor and leave one' do
    @board.favors << :bree
    @board.save

    put hero_exploration_url( @hero, tokens: { favor: [ :favor ] } )
    assert_redirected_to hero_exploration_url(@hero)
    assert_equal 1, @hero.reload.favor

    assert_equal %w( old_forest bree ).sort, @board.reload.favors.sort
  end

  test 'should get two favors and leave none' do
    @board.favors << :bree
    @board.save

    put hero_exploration_url( @hero, tokens: { favor: [ :favor, :favor ] } )
    assert_redirected_to hero_exploration_url(@hero)
    assert_equal 2, @hero.reload.favor

    assert_equal %w( old_forest ).sort, @board.reload.favors.sort
  end

  test 'should get only one favor because of the dispairing corruption card' do
    @board.favors << :bree
    @board.save

    @hero.favor = 2
    @hero.save!

    create( :dispairing, board: @board, hero: @hero )

    put hero_exploration_url( @hero, tokens: { favor: [ :favor, :favor ] } )
    assert_redirected_to hero_exploration_url(@hero)
    assert_equal 3, @hero.reload.favor

    assert_equal %w( bree old_forest ).sort, @board.reload.favors.sort

    get board_logs_url(@board)
    assert_select 'td', true, "Wasn't able to get all favors because of the dispairing corruption."

  end


end
