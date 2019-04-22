require 'test_helper'

class CombatCallOnlyTest < ActiveSupport::TestCase

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
