namespace :combats do

  class Hammer < Thor
    include Thor::Actions
  end

  desc 'Create a test from a stored combat in database'
  task :ceate_test, [:combat_id]  => :environment  do |t, args|
    args.with_defaults(:combat_id => nil)
    c = Combat.find( args.combat_id )

    data_to_insert = [ "\ttest 'Combat example ##{args.combat_id}' do" ]

    data_to_insert << "\t\tmob = create( :#{c.mob.code}, board: @board )"
    data_to_insert << "\t\t@board.combat.mob = mob"
    data_to_insert << "\t\t@board.combat.temporary_hero_strength = #{c.temporary_hero_strength}"
    data_to_insert << "\t\t@board.combat.save!"

    data_to_insert << "\tend"

    data_to_insert = data_to_insert.join( "\n" ) + "\n"

    Hammer.new.send :insert_into_file, 'test/models/combat_scenarios_test.rb', data_to_insert, :before => /^end/
  end
end