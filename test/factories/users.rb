FactoryBot.define do
  factory :user do
    provider { 'discord' }
    uid { 'foobar' }
    name { 'foo' }
    email { 'bar' }
  end
end
