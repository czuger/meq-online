FactoryBot.define do
  factory :monster do
    pool_key { 'monsters_pool_orange' }
    code { 'agent' }
    location { 'the_nindale' }

    fortitude { 8 }
    strength { 8 }
    life { 8 }

    name { 'Agent' }

    hand { [ 3, 4, 5 ] }

    attack_deck { :ravager }
  end
end
