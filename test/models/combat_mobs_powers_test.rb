require 'test_helper'

class CombatMobsPowersTest < ActiveSupport::TestCase

  setup do
    @game_data_heroes ||= GameData::Heroes.new
    @game_data_mobs_cards ||= GameData::MobsCards.new

    @user = create( :user )
    @board = create( :board )

    @hero = create( :hero, user: @user, board: @board )
    @mob = create( :orc, board: @board )
    create( :sauron, board: @board, user: @user )

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


      assert_difference '@hero.reload.temporary_damages', 1 do
        assert_difference '@mob.reload.life', 0 do
          @board.combat.reveal_secretly_played_cards
        end
      end

  end

  test 'snaga power' do
    @board.combat.hero_secret_played_card = 6
    @board.combat.mob_secret_played_card = 1
    @board.combat.save!

    mob = create( :snaga, board: @board )
    @board.combat.mob = mob
    @board.combat.save!


      assert_difference '@hero.reload.temporary_damages', 1 do
        assert_difference 'mob.reload.life', 0 do
          @board.combat.reveal_secretly_played_cards
        end
      end

  end

  test 'huorn power' do
    @board.combat.hero_secret_played_card = 6
    @board.combat.mob_secret_played_card = 9
    @board.combat.save!

    mob = create( :huorn, board: @board )
    @board.combat.mob = mob
    @board.combat.save!


      assert_difference '@hero.reload.temporary_damages', 3 do
        assert_difference 'mob.reload.life', -3 do
          @board.combat.reveal_secretly_played_cards
        end
      end

  end

  test 'cave troll power' do
    @board.combat.hero_secret_played_card = 2
    @board.combat.mob_secret_played_card = 1
    @board.combat.save!

    mob = create( :cave_troll, board: @board )
    @board.combat.mob = mob
    @board.combat.save!


      assert_difference '@hero.reload.temporary_damages', 3 do
        assert_difference 'mob.reload.life', 0 do
          @board.combat.reveal_secretly_played_cards
        end
      end

  end

  test 'southron power' do
    @board.combat.hero_secret_played_card = 2
    @board.combat.mob_secret_played_card = 10
    @board.combat.save!

    mob = create( :southron, board: @board )
    @board.combat.mob = mob
    @board.combat.save!


      assert_difference '@hero.reload.temporary_damages', 1 do
        assert_difference 'mob.reload.life', -2 do
          assert_difference '@board.combat.reload.mob_strength_used', 2 do
            @board.combat.reveal_secretly_played_cards
          end
        end
      end

  end

  test 'southron power assert zero strength cost' do
    @board.combat.hero_secret_played_card = 2
    @board.combat.mob_secret_played_card = 0
    @board.combat.save!

    mob = create( :southron, board: @board )
    @board.combat.mob = mob
    @board.combat.save!


      assert_difference '@hero.reload.temporary_damages', 0 do
        assert_difference 'mob.reload.life', -2 do
          assert_difference '@board.combat.reload.mob_strength_used', 0 do
            @board.combat.reveal_secretly_played_cards
          end
        end
      end

  end

end
