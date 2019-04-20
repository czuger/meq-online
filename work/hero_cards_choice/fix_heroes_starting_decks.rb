require 'yaml'

# given by :
# ( GameData::Heroes.new.common_cards_names + GameData::MobsCards.new.common_cards_names ).inject(Hash.new(0)) { |total, e| total[e] += 1 ;total}.select{ |e, i| i >= 3 }.map{ |e| e.first }.sort
KEPT_CARDS = [ 'Concentrate', 'Deadly Finesse', 'Overdraw', 'Precision', 'Aimed Shot', 'Attack of Opportunity', 'Charge', 'Evade', 'Fall Back', 'Hack', 'Parry', 'Ranged Strike', 'Reckless', 'Smash']

heroes_path = '../../app/models/game_data/heroes.yaml'

heroes = YAML.load_file( heroes_path )

heroes.each do |_, hero|
  hero_cards = []
  hero[:cards].each do |card_no, card_val|
    hero_cards << card_no if KEPT_CARDS.include?( card_val[:name].to_s )
  end

  cards = hero_cards.clone
  cards_pool = []
  while cards_pool.count < 25
    cards_pool << cards.shift
    cards = hero_cards.clone if cards.empty?
  end
  hero[:starting_deck] = cards_pool
end

File.open( heroes_path, 'w' ){ |f| f.puts heroes.to_yaml }