require 'test_helper'
require 'pp'

class GameEngineDeckTest < ActiveSupport::TestCase

  DECK_NAME = :shadow

  def setup
    @user = create( :user )
    @board = create( :board )
    @board.users << @user
    @sauron = create( :sauron, board: @board, user: @user )
  end

  test 'should refill deck if near empty' do
    @board.shadow_deck = [ 4 ]
    @board.shadow_discard = [ 10, 9, 8, 7, 6, 5 ]
    @board.save!

    @sauron.drawn_shadow_cards = []
    @sauron.save!

    ge = GameEngine::Deck.new(@user, @board, @sauron, DECK_NAME, discard_card_action: :discard )

    ge.draw_cards(3)

    assert_empty @sauron.reload.drawn_shadow_cards - [ 10, 9, 8, 7, 6, 5, 4 ]
    assert_equal 3, @sauron.reload.drawn_shadow_cards.count

    # pp @board.reload.shadow_deck
    assert_empty @board.reload.shadow_deck - [ 10, 9, 8, 7, 6, 5, 4 ]
    assert_equal 4, @board.reload.shadow_deck.count

  end

end