require 'pp'
require 'yaml'

def symbolize( name )
  name.gsub( /[-_ ]/, '_' ).downcase.to_sym
end

cards = {}
[ :behemoth, :ravager, :zealot ].each do |deck|
  File.open( "monsters/#{deck}/#{deck}.txt", 'r' ) do |card_file|
    card_id = 0
    card_file.readlines.each do |card_line|

      name, type, attack, defense, strength_cost, ability, card_quantity = card_line.split("\t")
      name_code = card_id
      card_quantity = card_quantity.to_i

      cards[deck] ||= { deck: [], data:{} }
      cards[deck][:data][name_code] = {
        name: name, type: symbolize( type ), attack: attack.to_i, defense: defense.to_i, strength_cost: strength_cost.to_i,
        pic_path: "#{deck}/#{symbolize( name )}.jpg" }
      cards[deck][:deck] += Array.new( card_quantity, name_code )
      card_id += 1
    end
  end
end

monsters = { monsters: {} }
File.open('monsters/monsters/monsters.txt','r') do |f|

  f.readlines.each do |line|
    name, fortitude, strength, wisdom, attack_deck = line.split("\t")
    name_code = symbolize( name )
    attack_deck = symbolize( attack_deck.chomp )

    monster = { name: name, fortitude: fortitude.to_i, strength: strength.to_i, wisdom: wisdom.to_i, attack_deck: attack_deck }
    # cards_deck = []
    #
    # cards_deck += Array.new( card_quantity, card_id )
    # card_id += 1

    monsters[:monsters][name_code] = monster

  end

  monsters[:cards] = cards

  File.open('../app/models/game_data/monsters.yaml','w') do |of|
    of.puts monsters.to_yaml
  end
end
