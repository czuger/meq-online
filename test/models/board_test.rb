require 'test_helper'

class BoardTest < ActiveSupport::TestCase

  test 'Create a board' do
    @board, _, _ = Board.create_new_board
    @board.save!
  end

end
