namespace :combats do

  class Hammer < Thor
    include Thor::Actions
  end

  # TODO : add a combat save pool, in order to save hero and mob data at the begigning of the combat
  desc 'Create a test from a stored combat in database'
  task :ceate_test, [:combat_id]  => :environment  do |t, args|
    args.with_defaults(:combat_id => nil)
    c = Combat.find( args.combat_id )

    data_to_insert = [ "\n", "\ttest 'Combat example ##{args.combat_id}' do" ]

    mob_hand = c.mob.hand + c.combat_card_played_mobs.map{ |e| e.card }
    data_to_insert << "\t\tmob = create( :#{c.mob.code}, board: @board, hand: #{mob_hand.inspect} )"
    data_to_insert << "\t\t@board.combat.mob = mob"

    hero_hand = c.hero.hand + c.combat_card_played_heroes.map{ |e| e.card }
    data_to_insert << "\t\thero = create( :#{c.hero.name_code}, board: @board, user: @user, hand: #{hero_hand.inspect} )"
    data_to_insert << "\t\t@board.combat.hero = hero"

    data_to_insert << "\t\t@board.combat.temporary_hero_strength = #{c.temporary_hero_strength}"
    data_to_insert << "\t\t@board.combat.save!"

    heroes_cards = c.combat_card_played_heroes.order( :id ).to_a
    mobs_cards = c.combat_card_played_mobs.order( :id ).to_a

    until heroes_cards.empty?
      hc = heroes_cards.shift
      mc = mobs_cards.shift

      data_to_insert << "\n\t\t@board.combat.hero_secret_played_card = #{hc.card}"
      data_to_insert << "\t\t@board.combat.mob_secret_played_card = #{mc.card}"

      data_to_insert << "\t\tassert_difference 'hero.reload.temporary_damages', 0 do"
      data_to_insert << "\t\t\tassert_difference 'mob.reload.life', 0 do"
      data_to_insert << "\t\t\t\t@board.combat.reveal_secretly_played_cards"
      data_to_insert << "\t\t\tend"
      data_to_insert << "\t\tend"
    end

    data_to_insert << "\tend\n"

    data_to_insert = data_to_insert.join( "\n" ) + "\n"

    Hammer.new.send :insert_into_file, 'test/models/combat_scenarios_test.rb', data_to_insert, :before => /^end/
  end
end