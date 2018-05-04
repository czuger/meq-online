require 'test_helper'
require 'pp'

class HeroesTest < ActiveSupport::TestCase

  def setup
    @heroes = GameData::Heroes.new
    @hero = @heroes.get( :argalad )
  end

  test 'should get hero' do
    assert @heroes.get( :argalad )
  end

  test 'should read hero fortitude' do
    assert @hero.fortitude

    # pp @hero
  end

end