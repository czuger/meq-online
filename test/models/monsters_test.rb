require 'test_helper'
require 'pp'

class MonstersTest < ActiveSupport::TestCase

  def setup
    @monsters = GameData::Mob.new
    @monster = @monsters.get( :crebain )
  end

  test 'should get monster' do
    assert @monsters.get( :crebain )
  end

  test 'should read fortitude' do
    assert @monster.fortitude

    # pp @monster
  end

end