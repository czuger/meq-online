FactoryBot.define do
  factory :monster do
    pool_key { 'monsters_pool_orange' }
    code { 'agent' }
    location { 'the_shire' }
  end
end
