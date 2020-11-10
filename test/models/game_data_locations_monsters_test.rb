require 'test_helper'
require 'pp'

class GameDataLocationsMonsters < ActiveSupport::TestCase

  def setup
    @board = create( :board )
    @lm = GameData::LocationsMonsters.new
    @loc = GameData::Locations.new
  end

  test 'test monster pick twice on the shire' do
    m = nil

    assert_difference '@board.reload.monsters_pool_orange.length', -1 do
      m = @lm.place_new_monster(@board, :the_shire)
    end

    assert_equal 'monsters_pool_orange', m.pool_key

    assert_difference '@board.reload.monsters_pool_orange.length', -1 do
      m = @lm.place_new_monster(@board, :the_shire)
    end

    assert_equal 'monsters_pool_orange', m.pool_key
  end


  test 'test monster pick twice on near_harad' do
    m = nil

    assert_difference '@board.reload.monsters_pool_dark_blue.length', -1 do
      m = @lm.place_new_monster(@board, :near_harad)
    end

    assert_equal 'monsters_pool_dark_blue', m.pool_key

    assert_difference '@board.reload.monsters_pool_dark_blue.length', -1 do
      m = @lm.place_new_monster(@board, :near_harad)
    end

    assert_equal 'monsters_pool_dark_blue', m.pool_key
  end

  test 'test monster pick on all locations' do
    @loc.all_codes.sort.each do |loc|
      m = @lm.place_new_monster(@board, loc)
      @lm.place_monster_back_to_monster_pool(@board, m )
    end
  end


end