require 'test_helper'
require 'pp'

class MonstersTest < ActiveSupport::TestCase

  def setup
    @heroes = GameData::Heroes.new
    @hero = @heroes.get( :argalad )
  end

  test 'should get monster' do
    assert @heroes.get( :argalad )
  end

  test 'should read fortitude' do
    assert @hero.fortitude

    pp @hero
  end

end