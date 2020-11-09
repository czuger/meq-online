require 'test_helper'

class BoardTest < ActiveSupport::TestCase

  test 'Create a board' do
    @board = Board.create_new_board
    @board.save!
  end

end
