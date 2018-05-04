require 'pp'
require 'yaml'

def symbolize( name )
  name.gsub( /[-_ ]/, '_' ).downcase.to_sym
end

hero = %w( argalad beravor eleanor eometh thalin )

heroes = YAML.load_file( 'hero_data/heroes_list.yaml' )

hero.each do |hero_name|

  hero_card_id = 0
  hero_cards = {}
  hero_starting_deck = []

  File.open("hero_data/#{hero_name}.txt",'r') do |f|

    f.readlines.each do |line|

      name, type, attack, defense, strength_cost, movement, _, card_qtt = line.split("\t")
      name_code = name.gsub(' ','_').downcase

      pic_path = "#{hero_name}/#{name_code}.jpg"
      card_image_path = 'hero_cards/' + pic_path

      if File.exists?(card_image_path)

        card = { name: name.to_sym, type: type.downcase.to_sym, attack: attack.to_i, defense: defense.to_i,
                 strength_cost: strength_cost.to_i, movement_type: movement.downcase.to_sym, pic_path: pic_path }
        hero_cards[hero_card_id] = card

        hero_starting_deck += 1.upto(card_qtt.to_i).map{ hero_card_id }
      else
        puts "#{card_image_path} does not exist."
        exit
      end

      hero_card_id += 1
    end

    # p hero_name
    # pp heroes[hero_name.to_sym]
    heroes[hero_name.to_sym][:starting_deck] = hero_starting_deck
    heroes[hero_name.to_sym][:cards] = hero_cards

  end
end

File.open('../app/models/game_data/heroes.yaml','w') do |f|
  f.puts heroes.to_yaml
end