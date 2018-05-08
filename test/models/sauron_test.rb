require 'test_helper'

class SauronTest < ActiveSupport::TestCase

  test 'Sauron name is Sauron' do
    user = create( :user )
    board = create( :board )
    sauron = create( :sauron, board: board, user: user )
    assert_equal 'Sauron', sauron.name
  end

end
