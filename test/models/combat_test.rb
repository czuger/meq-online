require 'test_helper'

class CombatTest < ActiveSupport::TestCase

  setup do
    @game_data_heroes ||= GameData::Heroes.new
    @game_data_mobs_cards ||= GameData::MobsCards.new

    @user = create( :user )
    @board = create( :board )

    @hero = create( :hero, user: @user, board: @board, hand: @game_data_heroes.get_deck(:argalad ),
      life_pool: @game_data_heroes.get_deck(:argalad ) )
    @mob = create( :monster, board: @board, hand: @game_data_mobs_cards.get_deck( 'ravager' ) )

    @board.create_combat( @hero, @mob )

    @board.combat.hero_secret_played_card = @hero.hand.first
    @board.combat.mob_secret_played_card = @mob.hand.first
    @board.combat.save!

    @board.users << @user
  end

  test 'attack of opportunity vs charge' do
    @board.combat.hero_secret_played_card = 8
    @board.combat.mob_secret_played_card = 8
    @board.combat.save!

    @mob.attack_deck = 'zealot'
    @mob.hand = @game_data_mobs_cards.get_deck( 'zealot' )
    @mob.save!

    assert_difference '@hero.reload.life_pool.count', -2 do
      assert_difference '@hero.damage_pool.count', 2 do
        assert_difference '@mob.reload.life', -7 do
          @board.combat.reveal_secretly_played_cards
        end
      end
    end
  end

  test 'charge vs aimed shot' do
    @board.combat.hero_secret_played_card = 3
    @board.combat.mob_secret_played_card = 7
    @board.combat.save!

    assert_difference '@hero.reload.life_pool.count', -2 do
      assert_difference '@hero.damage_pool.count', 2 do
        assert_no_difference '@mob.reload.life' do
          @board.combat.reveal_secretly_played_cards
        end
      end
    end
  end

  test 'fall_back vs attack of_opportunity, then parry vs attack of_opportunity' do
    @board.combat.hero_secret_played_card = 4
    @board.combat.mob_secret_played_card = 10
    @board.combat.save!

    assert_difference '@hero.reload.life_pool.count', -1 do
      assert_difference '@hero.damage_pool.count', 1 do
        assert_difference '@mob.reload.life', -1 do
          @board.combat.reveal_secretly_played_cards
        end
      end
    end

    @board.combat.hero_secret_played_card = 6
    @board.combat.mob_secret_played_card = 10
    @board.combat.save!

    assert_no_difference '@hero.reload.life_pool.count' do
      assert_no_difference '@hero.damage_pool.count' do
        assert_difference '@mob.reload.life', -1 do
          @board.combat.reveal_secretly_played_cards
        end
      end
    end

  end

  test 'Call all cards for ravager' do
    @mob.hand.each do |card|
      @board.combat.mob_secret_played_card = card
      @board.combat.save!
      @board.combat.reveal_secretly_played_cards
    end
  end

  test 'Call all cards for zealot' do
    @mob.attack_deck = 'zealot'
    @mob.save!
    @game_data_mobs_cards.get_deck( 'zealot' ).each do |card|
      @board.combat.mob_secret_played_card = card
      @board.combat.save!
      @board.combat.reveal_secretly_played_cards
    end
  end

  test 'Call all cards for behemoth' do
    @mob.attack_deck = 'behemoth'
    @mob.save!
    @game_data_mobs_cards.get_deck( 'behemoth' ).each do |card|
      @board.combat.mob_secret_played_card = card
      @board.combat.save!
      @board.combat.reveal_secretly_played_cards
    end
  end

  test 'Call all cards for argalad' do
    @game_data_heroes.get_deck( :argalad ).each do |card|
      @board.combat.hero_secret_played_card = card
      @board.combat.save!
      @board.combat.reveal_secretly_played_cards
    end
  end

  test 'Call all cards for beravor' do
    @hero.name_code = 'beravor'
    @hero.save!
    @game_data_heroes.get_deck( :beravor ).each do |card|
      @board.combat.hero_secret_played_card = card
      @board.combat.save!
      @board.combat.reveal_secretly_played_cards
    end
  end

  test 'Call all cards for eleanor' do
    @hero.name_code = 'eleanor'
    @hero.save!
    @game_data_heroes.get_deck( :eleanor ).each do |card|
      @board.combat.hero_secret_played_card = card
      @board.combat.save!
      @board.combat.reveal_secretly_played_cards
    end
  end

  test 'Call all cards for eometh' do
    @hero.name_code = 'eometh'
    @hero.save!
    @game_data_heroes.get_deck( :eometh ).each do |card|
      @board.combat.hero_secret_played_card = card
      @board.combat.save!
      @board.combat.reveal_secretly_played_cards
    end
  end

  test 'Call all cards for thalin' do
    @hero.name_code = 'thalin'
    @hero.save!
    @game_data_heroes.get_deck( :thalin ).each do |card|
      @board.combat.hero_secret_played_card = card
      @board.combat.save!
      @board.combat.reveal_secretly_played_cards
    end
  end

end
