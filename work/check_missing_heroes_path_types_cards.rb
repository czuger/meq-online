require 'yaml'

heroes = YAML.load_file('../app/models/game_data/heroes.yaml')
required_movement_types = [:hills, :forest, :sun, :mountains, :desolation]

heroes.each do |h_name, h_data|
  movement_types = []
  h_data[:starting_deck].uniq.each do |card_no|
    card_data = h_data[:cards][card_no]
    movement_types << card_data[:movement_type] unless movement_types.include?(card_data[:movement_type])
  end
  missing_movement_types = required_movement_types - movement_types
  puts h_name
  h_data[:cards].each do |card_id, card_value|
    puts "#{card_value[:name]} - #{card_value[:movement_type]}" if missing_movement_types.include?(card_value[:movement_type])
  end
  puts
end