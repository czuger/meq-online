require 'test_helper'

class CorruptionTest < ActiveSupport::TestCase

  def setup
    @user = create( :user )
    @board = create( :board )
    @hero = create( :hero, user: @user, board: @board, location: :the_ruins_of_angmar )

    Kernel.stubs( :rand ).returns( 1 )
  end

  test 'On a roll of 5, nothing should happen' do
    Kernel.stubs( :rand ).returns( 5 )

    assert_no_difference 'Corruption.count' do
      @hero.suffer_peril(@board)
    end
  end

  test 'modifications_cards' do
    cards = [ [ 8, :wisdom ], [ 2, :agility ], [ 7, :fortitude ], [ 14, :strength ] ]

    cards.each do |c|
      @board.corruption_deck = [c.first]
      @board.save!

      assert_difference 'Corruption.count' do
        assert_difference "@hero.reload.#{c.last}", -1 do
          @hero.suffer_peril(@board)
          @hero.save!
        end
      end
    end
  end

  # test 'The shire should be perilous' do
  #   Kernel.stubs( :rand ).returns( 3 )
  #
  #   @hero.location = :the_shire
  #   @hero.wisdom = 2
  #   @hero.save!
  #
  #   @board.influence[:the_shire] = 3
  #   @board.save!
  #
  #   assert_difference '@hero.reload.favor', -1 do
  #     @hero.suffer_peril!(@board)
  #   end
  # end

  #
  # test 'On a roll of 2, should loose a card' do
  #   Kernel.stubs( :rand ).returns( 2 )
  #
  #   assert_difference '@hero.reload.hand.count', -1 do
  #     assert_difference '@hero.reload.damage_pool.count' do
  #       @hero.suffer_peril!(@board)
  #     end
  #   end
  # end
  #
  # test 'If a hero has no more cards, then he loose nothing' do
  #   Kernel.stubs( :rand ).returns( 2 )
  #
  #   @hero.hand = []
  #   @hero.save!
  #
  #   assert_no_difference '@hero.reload.hand.count', -1 do
  #     assert_no_difference '@hero.reload.damage_pool.count' do
  #       @hero.suffer_peril!(@board)
  #     end
  #   end
  # end

  # test 'On a roll of 3, should loose a favor' do
  #   Kernel.stubs( :rand ).returns( 3 )
  #
  #   assert_difference '@hero.reload.favor', -1 do
  #     @hero.suffer_peril!(@board)
  #   end
  # end
  #
  # test 'On a roll of 4, should loose a favor and a card' do
  #   Kernel.stubs( :rand ).returns( 4 )
  #
  #   assert_difference '@hero.reload.hand.count', -1 do
  #     assert_difference '@hero.reload.damage_pool.count' do
  #       assert_difference '@hero.reload.favor', -1 do
  #         @hero.suffer_peril!(@board)
  #       end
  #     end
  #   end
  # end

end
