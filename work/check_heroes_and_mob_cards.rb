require 'yaml'

`rm heroes_and_mobs_cards/*`

heroes = YAML.load_file('../app/models/game_data/heroes.yaml')

heroes.each do |h_name, h_data|
  h_data[:starting_deck].uniq.each do |card_no|
    card_path = "../app/assets/images/hero_cards/actions/#{h_data[:cards][card_no][:pic_path]}"
    if File.exists? card_path
      new_filename = File.basename(card_path, '.jpg') + '_' + h_name.to_s + '.jpg'
          `cp #{card_path} heroes_and_mobs_cards/#{new_filename}`
    end
  end
end

mobs = YAML.load_file('../app/models/game_data/mobs_cards.yaml')

mobs.each do |h_name, h_data|
  h_data[:deck].uniq.each do |card_no|
    card_path = "../app/assets/images/monsters/#{h_data[:data][card_no][:pic_path]}"
    if File.exists? card_path
      new_filename = File.basename(card_path, '.jpg') + '_' + h_name.to_s + '.jpg'
      `cp #{card_path} heroes_and_mobs_cards/#{new_filename}`
    end
  end
end

