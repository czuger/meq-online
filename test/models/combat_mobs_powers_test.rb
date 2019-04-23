require 'test_helper'

class CombatScenariosTest < ActiveSupport::TestCase

  setup do
    @game_data_heroes ||= GameData::Heroes.new
    @game_data_mobs_cards ||= GameData::MobsCards.new

    @user = create( :user )
    @board = create( :board )

    @hero = create( :hero, user: @user, board: @board, hand: @game_data_heroes.get_deck(:argalad ),
      life_pool: @game_data_heroes.get_deck(:argalad ) )
    @mob = create( :orc, board: @board, hand: @game_data_mobs_cards.get_deck( 'zealot' ) )

    @board.create_combat( @hero, @mob )

    @board.combat.temporary_hero_strength = 5
    @board.combat.hero_secret_played_card = @hero.hand.first
    @board.combat.mob_secret_played_card = @mob.hand.first
    @board.combat.save!

    @board.users << @user
  end

  test 'orc power' do
    @board.combat.hero_secret_played_card = 6
    @board.combat.mob_secret_played_card = 9
    @board.combat.save!

    assert_difference '@hero.reload.life_pool.count', -1 do
      assert_difference '@hero.damage_pool.count', 1 do
        assert_difference '@mob.reload.life', 0 do
          @board.combat.reveal_secretly_played_cards
        end
      end
    end
  end

end
