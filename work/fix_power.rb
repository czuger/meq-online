require 'yaml'

heroes = YAML.load_file( '../app/models/game_data/heroes.yaml' )

heroes.each do |k, v|
  v[:cards].each do |kk, vv|
    vv[:power] = vv[:pic_path][0..-5].split('/').last
  end
end

File.open( '../app/models/game_data/heroes.yaml', 'w' ) { |f| f.write heroes.to_yaml } #Store

#
#
#

mobs_cards = YAML.load_file( '../app/models/game_data/mobs_cards.yaml' )

mobs_cards.each do |k, v|
  v[:data_and_rules].each do |kk, vv|
    vv[:power] = vv[:pic_path][0..-5].split('/').last
  end
end

File.open( '../app/models/game_data/mobs_cards.yaml', 'w' ) { |f| f.write mobs_cards.to_yaml } #Store