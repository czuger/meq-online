require 'test_helper'
require 'pp'

class GameDataLocationsMonsters < ActiveSupport::TestCase

  def setup
    @board = create( :board )
    @lm = GameData::LocationsMonsters.new
    @loc = GameData::Locations.new

    @lm.fill_board(@board)
  end

  test 'test monster pick twice on the shire' do
    m = nil

    assert_difference '@board.reload.monsters_pool_orange.length', -1 do
      m = @lm.pick_monster_from_board(@board, :the_shire)
    end

    assert_equal 'monsters_pool_orange', m.pool_key

    assert_difference '@board.reload.monsters_pool_orange.length', -1 do
      m = @lm.pick_monster_from_board(@board, :the_shire)
    end

    assert_equal 'monsters_pool_orange', m.pool_key
  end


  test 'test monster pick twice on near_harad' do
    m = nil

    assert_difference '@board.reload.monsters_pool_dark_blue.length', -1 do
      m = @lm.pick_monster_from_board(@board, :near_harad)
    end

    assert_equal 'monsters_pool_dark_blue', m.pool_key

    assert_difference '@board.reload.monsters_pool_dark_blue.length', -1 do
      m = @lm.pick_monster_from_board(@board, :near_harad)
    end

    assert_equal 'monsters_pool_dark_blue', m.pool_key
  end

  test 'test monster pick on all locations' do
    @loc.data.keys.sort.each do |loc|
      m = @lm.pick_monster_from_board(@board, loc)
      @lm.place_monster_back_to_monster_pool(@board, m )
    end
  end


end