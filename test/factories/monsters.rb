FactoryBot.define do
  factory :monster do
    pool_key { 'monsters_pool_orange' }
    code { 'agent' }
    location { 'the_shire' }

    fortitude { 8 }
    strength { 8 }
    life { 8 }

    name { 'Agent' }
  end
end
