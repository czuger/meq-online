FactoryBot.define do
  factory :user do
    provider 'developer'
    uid 123456789
    name 'foo'
    email 'bar'
  end
end
