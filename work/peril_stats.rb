require 'hazard'

results = { true: 0, false: 0 }

def three_cards
  [ one_card, one_card, one_card ].any?
end

def one_card
  case Hazard.d3
    when 1
      true
    when 2
      false
    else
      Hazard.d3 == 1 ? true : false
  end
end

p results
1.upto(1000000).each do
  results[three_cards.inspect.to_sym] += 1
end

puts "#{ ( results[:false]*100 ) / results[:true] }"

# Result : 20%


