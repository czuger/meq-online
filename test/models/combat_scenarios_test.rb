require 'test_helper'

class CombatScenariosTest < ActiveSupport::TestCase

  setup do
    @game_data_heroes ||= GameData::Heroes.new
    @game_data_mobs_cards ||= GameData::MobsCards.new

    @user = create( :user )
    @board = create( :board )

    @hero = create( :hero, user: @user, board: @board, hand: @game_data_heroes.get_deck(:argalad ),
      life_pool: @game_data_heroes.get_deck(:argalad ) )
    @mob = create( :southron, board: @board )
    create( :sauron, board: @board, user: @user )

    @board.create_combat( @hero, @mob )

    @board.combat.temporary_hero_strength = @hero.strength + @hero.agility
    @board.combat.hero_secret_played_card = @hero.hand.first
    @board.combat.mob_secret_played_card = @mob.hand.first
    @board.combat.save!

    @board.users << @user
  end

  test 'precision vs ranged strike' do
    @board.combat.hero_secret_played_card = 2
    @board.combat.mob_secret_played_card = 1
    @board.combat.save!

    @mob.attack_deck = 'zealot'
    @mob.hand = @game_data_mobs_cards.get_deck( 'zealot' )
    @mob.save!


      assert_difference '@hero.reload.temporary_damages', 2 do
        assert_difference '@mob.reload.life', -2 do
          @board.combat.reveal_secretly_played_cards
        end
      end

  end

  test 'attack of opportunity vs charge' do
    @board.combat.hero_secret_played_card = 8
    @board.combat.mob_secret_played_card = 8
    @board.combat.save!

    @mob.attack_deck = 'zealot'
    @mob.hand = @game_data_mobs_cards.get_deck( 'zealot' )
    @mob.save!


      assert_difference '@hero.reload.temporary_damages', 2 do
        assert_difference '@mob.reload.life', -7 do
          @board.combat.reveal_secretly_played_cards
        end
      end

  end

  test 'charge vs aimed shot' do
    @board.combat.hero_secret_played_card = 3
    @board.combat.mob_secret_played_card = 7
    @board.combat.save!


      assert_difference '@hero.reload.temporary_damages', 2 do
        assert_no_difference '@mob.reload.life' do
          @board.combat.reveal_secretly_played_cards
        end
      end

  end

  test 'fall_back vs attack of_opportunity, then parry vs attack of_opportunity' do
    @board.combat.hero_secret_played_card = 4
    @board.combat.mob_secret_played_card = 10
    @board.combat.save!

    @mob.strength = 50
    @mob.save!


      assert_difference '@hero.reload.temporary_damages', 1 do
        assert_difference '@mob.reload.life', -1 do
          @board.combat.reveal_secretly_played_cards
        end
      end


    @board.combat.hero_secret_played_card = 6
    @board.combat.mob_secret_played_card = 10
    @board.combat.save!


      assert_no_difference '@hero.reload.temporary_damages' do
        assert_difference '@mob.reload.life', -1 do
          @board.combat.reveal_secretly_played_cards
        end
      end

  end

  test 'Mouth of Sauron example 1' do
    mob = create( :mouth_of_sauron, board: @board )
    @board.combat.mob = mob

    @board.combat.temporary_hero_strength = @hero.strength

    @board.combat.save!

    @board.combat.hero_secret_played_card = @game_data_heroes.get_card_number_by_name( :argalad, 'Volley')
    @board.combat.mob_secret_played_card = @game_data_mobs_cards.get_card_number_by_name( 'zealot', 'Ranged Strike')
    @board.combat.save!


      assert_difference '@hero.reload.temporary_damages', 1 do
        assert_difference 'mob.reload.life', -1 do
          @board.combat.reveal_secretly_played_cards
        end
      end


    @board.combat.hero_secret_played_card = @game_data_heroes.get_card_number_by_name( :argalad, 'Aimed Shot')
    @board.combat.mob_secret_played_card = @game_data_mobs_cards.get_card_number_by_name( 'zealot', 'Reckless')
    @board.combat.save!


      assert_difference '@hero.reload.temporary_damages', 5 do
        assert_difference 'mob.reload.life', 0 do
          @board.combat.reveal_secretly_played_cards
        end
      end

  end

end
