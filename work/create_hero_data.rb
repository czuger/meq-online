require 'pp'
require 'yaml'

hero = %w( argalad beravor elanor eometh thalin skills )

hero.each do |hero_name|

  hero_card_id = 0
  hero_cards = {}
  hero_decks = []

  File.open("hero_data/#{hero_name}.txt",'r') do |f|

    f.readlines.each do |line|
      name, type, attack, defense, strength_cost, movement, _, card_qtt = line.split("\t")
      name = name.gsub(' ','_').downcase
      card_image_path = "hero_cards/#{hero_name}/#{name}.jpg"
      if File.exists?(card_image_path)
        card = { name: name.to_sym, type: type.downcase.to_sym, attack: attack, defense: defense, strength_cost: strength_cost,
                 movement_type: movement.downcase.to_sym }
        hero_cards[hero_card_id] = card

        hero_decks += 1.upto(card_qtt.to_i).map{ hero_card_id }
      else
        puts "#{card_image_path} does not exist."
        exit
      end

      hero_card_id += 1
    end

    File.open("../app/models/data/heros/#{hero_name}_actions_cards.yaml",'w') do |f|
      f.puts hero_cards.to_yaml
    end

    File.open("../app/models/data/heros/#{hero_name}_starting_deck.yaml",'w') do |f|
      f.puts hero_decks.to_yaml
    end
  end
end