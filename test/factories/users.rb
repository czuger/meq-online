FactoryBot.define do
  factory :user do
    provider { 'google_oauth2' }
    uid { 1 }
    name { 'foo' }
    email { 'bar' }
  end
end
