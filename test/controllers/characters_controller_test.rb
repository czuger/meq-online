require 'test_helper'

class CharactersControllerTest < ActionDispatch::IntegrationTest

  setup do
    @user = create( :user )
    @board = create( :board )
    @hero = create( :hero, user: @user, board: @board )
    @board.users << @user

    connection_for_tests
  end

  teardown do
    OmniAuth.config.test_mode = false
  end

  test 'should get edit' do
    get edit_character_url @hero
    assert_response :success
  end

  test 'should add character' do
    put character_url @hero, params: { characters: { aragorn: :blue_mountains } }
    assert_redirected_to edit_character_url @hero
  end

  test 'should remove character' do
    @board.characters['aragorn'] = 'blue_mountains'
    @board.save!
    put character_url @hero, params: { characters: { aragorn: '' } }
    assert_redirected_to edit_character_url @hero
  end

end
