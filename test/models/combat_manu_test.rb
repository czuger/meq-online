require 'test_helper'

class CombatManuTest < ActiveSupport::TestCase

  setup do
    @game_data_heroes ||= GameData::Heroes.new
    @game_data_mobs_cards ||= GameData::MobsCards.new

    @user = create( :user )
    @board = create( :board )

    @hero = create( :hero, user: @user, board: @board, hand: @game_data_heroes.get_deck(:argalad ),
      life_pool: @game_data_heroes.get_deck(:argalad ) )
    @mob = create( :monster, board: @board, hand: @game_data_mobs_cards.get_deck( 'zealot' ),
                   attack_deck: :zealot, strength: 50, life: 50 )
    create( :sauron, board: @board, user: @user )

    @board.create_combat( @hero, @mob )

    @board.combat.temporary_hero_strength = 50
    @board.combat.hero_secret_played_card = @hero.hand.first
    @board.combat.mob_secret_played_card = @mob.hand.first
    @board.combat.save!

    @board.users << @user
  end

  test 'Combat 4' do
    mob = create( :snaga, board: @board )
    @board.combat.mob = mob

    @board.combat.temporary_hero_strength = @hero.strength

    @board.combat.hero_secret_played_card = @game_data_heroes.get_card_number_by_name( :argalad, 'Precision')
    @board.combat.mob_secret_played_card = @game_data_mobs_cards.get_card_number_by_name( 'zealot', 'Hack')
    @board.combat.save!

    assert_difference '@hero.reload.temporary_damages', 0 do
      assert_difference 'mob.reload.life', -2 do
        @board.combat.reveal_secretly_played_cards
      end
    end
  end

  test 'Combat 3' do
    @hero.name_code = 'beravor'
    @hero.hand = @game_data_heroes.get_deck(:beravor )
    @hero.save!

    @board.combat.hero_secret_played_card = 0
    @board.combat.mob_secret_played_card = 4
    @board.combat.save!

    hero_damages = -0

      assert_difference '@hero.reload.temporary_damages', -hero_damages do
        assert_difference '@mob.reload.life', -2 do
          @board.combat.reveal_secretly_played_cards
        end
      end

    @board.combat.hero_secret_played_card = 6
    @board.combat.mob_secret_played_card = 7
    @board.combat.save!

    hero_damages = -1

      assert_difference '@hero.reload.temporary_damages', -hero_damages do
        assert_difference '@mob.reload.life', 0 do
          @board.combat.reveal_secretly_played_cards
        end
      end


    @board.combat.hero_secret_played_card = 4
    @board.combat.mob_secret_played_card = 10
    @board.combat.save!

    hero_damages = -5

      assert_difference '@hero.reload.temporary_damages', -hero_damages do
        assert_difference '@mob.reload.life', -8 do
          @board.combat.reveal_secretly_played_cards
        end
      end

  end

  test 'Combat 2' do
    @hero.name_code = 'beravor'
    @hero.hand = @game_data_heroes.get_deck(:beravor )
    @hero.save!

    @board.combat.hero_secret_played_card = 5
    @board.combat.mob_secret_played_card = 10
    @board.combat.save!

    hero_damages = -2

      assert_difference '@hero.reload.temporary_damages', -hero_damages do
        assert_difference '@mob.reload.life', -3 do
          @board.combat.reveal_secretly_played_cards
        end
      end


    @hero.name_code = 'argalad'
    @hero.hand = @game_data_heroes.get_deck(:argalad )
    @hero.save!

    @board.combat.hero_secret_played_card = 4
    @board.combat.mob_secret_played_card = 6
    @board.combat.save!

    hero_damages = 0

      assert_difference '@hero.reload.temporary_damages', -hero_damages do
        assert_difference '@mob.reload.life', 0 do
          @board.combat.reveal_secretly_played_cards
        end
      end


    @board.combat.hero_secret_played_card = 6
    @board.combat.mob_secret_played_card = 7
    @board.combat.save!

    hero_damages = 0

      assert_difference '@hero.reload.temporary_damages', -hero_damages do
        assert_difference '@mob.reload.life', -1 do
          @board.combat.reveal_secretly_played_cards
        end
      end

  end

  test 'Combat 1' do
    @board.combat.hero_secret_played_card = 8
    @board.combat.mob_secret_played_card = 10
    @board.combat.save!

    hero_damages = -5

      assert_difference '@hero.reload.temporary_damages', -hero_damages do
        assert_difference '@mob.reload.life', -9 do
          @board.combat.reveal_secretly_played_cards
        end
      end


    @hero.name_code = 'beravor'
    @hero.hand = @game_data_heroes.get_deck(:beravor )
    @hero.save!

    @board.combat.hero_secret_played_card = 1
    @board.combat.mob_secret_played_card = 10
    @board.combat.save!

    hero_damages = 0

      assert_difference '@hero.reload.temporary_damages', -hero_damages do
        assert_difference '@mob.reload.life', 0 do
          @board.combat.reveal_secretly_played_cards
        end
      end


    @board.combat.hero_secret_played_card = 0
    @board.combat.mob_secret_played_card = 9
    @board.combat.save!

    hero_damages = -2

      assert_difference '@hero.reload.temporary_damages', -hero_damages do
        assert_difference '@mob.reload.life', 0 do
          @board.combat.reveal_secretly_played_cards
        end
      end

  end

end
