require 'yaml'

# given by :
# ( GameData::Heroes.new.common_cards_names + GameData::MobsCards.new.common_cards_names ).inject(Hash.new(0)) { |total, e| total[e] += 1 ;total}.select{ |e, i| i >= 3 }.map{ |e| e.first }.sort
KEPT_CARDS = [ 'Concentrate', 'Deadly Finesse', 'Overdraw', 'Precision', 'Aimed Shot', 'Attack of Opportunity', 'Charge', 'Evade', 'Fall Back', 'Hack', 'Parry', 'Ranged Strike', 'Reckless', 'Smash']

mobs_path = '../../app/models/game_data/mobs_cards.yaml'

mobs = YAML.load_file( mobs_path )

mobs.each do |_, hero|
  hero_cards = []
  hero[:data].each do |card_no, card_val|
    hero_cards << card_no if KEPT_CARDS.include?( card_val[:name].to_s )
  end

  cards = hero_cards.clone
  cards_pool = []
  while cards_pool.count < 25
    cards_pool << cards.shift
    cards = hero_cards.clone if cards.empty?
  end
  hero[:deck] = cards_pool
end

File.open( mobs_path, 'w' ){ |f| f.puts mobs.to_yaml }