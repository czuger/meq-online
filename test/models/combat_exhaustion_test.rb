require 'test_helper'

class CombatExhaustionTest < ActiveSupport::TestCase

  setup do
    @game_data_heroes ||= GameData::Heroes.new
    @game_data_mobs_cards ||= GameData::MobsCards.new

    @user = create( :user )
    @board = create( :board )

    @hero = create( :hero, user: @user, board: @board, hand: @game_data_heroes.get_deck(:argalad ),
      life_pool: @game_data_heroes.get_deck(:argalad ) )
    @mob = create( :monster, board: @board, hand: @game_data_mobs_cards.get_deck( 'ravager' ) )

    @board.create_combat( @hero, @mob )

    @board.combat.temporary_hero_strength = 5
    @board.combat.hero_secret_played_card = @hero.hand.first
    @board.combat.mob_secret_played_card = @mob.hand.first
    @board.combat.save!

    @board.users << @user
  end

  test 'exhaustion test, both - aimed shot vs reckless on near' do
    @board.combat.hero_strength_used = 3
    @board.combat.mob_strength_used = 7

    @board.combat.hero_secret_played_card = 3
    @board.combat.mob_secret_played_card = 10
    @board.combat.save!

    @mob.attack_deck = 'zealot'
    @mob.hand = @game_data_mobs_cards.get_deck( 'zealot' )
    @mob.save!

    assert_no_difference '@hero.reload.life_pool.count' do
      assert_no_difference '@hero.damage_pool.count' do
        assert_no_difference '@mob.reload.life' do
          @board.combat.reveal_secretly_played_cards
        end
      end
    end

    assert @board.combat.combat_card_played_heroes.last.cancelled = true
  end

  test 'exhaustion test, hero only - aimed shot vs reckless on near' do
    @board.combat.hero_strength_used = 3
    @board.combat.mob_strength_used = 0

    @board.combat.hero_secret_played_card = 3
    @board.combat.mob_secret_played_card = 10
    @board.combat.save!

    @mob.attack_deck = 'zealot'
    @mob.hand = @game_data_mobs_cards.get_deck( 'zealot' )
    @mob.save!

    assert_difference '@hero.reload.life_pool.count', -5 do
      assert_difference '@hero.damage_pool.count', 5 do
        assert_no_difference '@mob.reload.life' do
          @board.combat.reveal_secretly_played_cards
        end
      end
    end

    assert @board.combat.combat_card_played_heroes.last.cancelled = true
  end

  test 'exhaustion test, mob only - aimed shot vs reckless on near' do
    @board.combat.hero_strength_used = 0
    @board.combat.mob_strength_used = 7

    @board.combat.hero_secret_played_card = 3
    @board.combat.mob_secret_played_card = 10
    @board.combat.save!

    @mob.attack_deck = 'zealot'
    @mob.hand = @game_data_mobs_cards.get_deck( 'zealot' )
    @mob.save!

    assert_no_difference '@hero.reload.life_pool.count' do
      assert_no_difference '@hero.damage_pool.count' do
        assert_difference '@mob.reload.life', -3 do
          @board.combat.reveal_secretly_played_cards
        end
      end
    end

    assert @board.combat.combat_card_played_heroes.last.cancelled = true
  end

end
