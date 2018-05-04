require 'pp'
require 'yaml'

def symbolize( name )
  name.gsub( /[-_ ]/, '_' ).downcase.to_sym
end

skill_card_id = 0
skill_cards = {}

File.open('skills_cards/skills.txt','r') do |f|

  f.readlines.each do |line|

    name, type, attack, defense, strength_cost, movement, _, card_qtt = line.split("\t")
    name_code = name.gsub(' ','_').downcase

    pic_path = "skills/#{name_code}.jpg"
    card_image_path = 'skills_cards/' + pic_path

    if File.exists?(card_image_path)

      card = { name: name.to_sym, type: type.downcase.to_sym, attack: attack.to_i, defense: defense.to_i,
               strength_cost: strength_cost.to_i, movement_type: movement.downcase.to_sym, pic_path: pic_path }
      skill_cards[skill_card_id] = card

    else
      puts "#{card_image_path} does not exist."
      exit
    end

    skill_card_id += 1
  end

end

File.open('../app/models/game_data/skills.yaml','w') do |f|
  f.puts skill_cards.to_yaml
end