require 'test_helper'

class CombatTest < ActiveSupport::TestCase

  setup do
    @game_data_heroes ||= GameData::Heroes.new
    @game_data_mobs_cards ||= GameData::MobsCards.new

    @user = create( :user )
    @board = create( :board )

    @hero = create( :hero, user: @user, board: @board, hand: @game_data_heroes.get_deck(:argalad ) )
    @mob = create( :monster, board: @board, hand: @game_data_mobs_cards.get_deck( 'ravager' ) )

    @board.create_combat( @hero, @mob )

    @board.combat.hero_secret_played_card = @hero.hand.first
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

end
