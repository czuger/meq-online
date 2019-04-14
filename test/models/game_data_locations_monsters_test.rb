require 'test_helper'
require 'pp'

class GameDataLocationsMonsters < ActiveSupport::TestCase

  def setup
    @board = create( :board )
    @lm = GameData::LocationsMonsters.new

    @lm.fill_board(@board)
  end

  test 'check if all event data is correct' do
    m = nil

    assert_difference '@board.reload.monsters_pool_orange.length', -1 do
      m = @lm.pick_monster_from_board(@board, :the_shire)
    end

    assert_equal 'monsters_pool_orange', m[:monster_pool_key]

    assert_difference '@board.reload.monsters_pool_orange.length', -1 do
      m = @lm.pick_monster_from_board(@board, :the_shire)
    end

    assert_equal 'monsters_pool_orange', m[:monster_pool_key]
  end
end