FactoryBot.define do
  factory :board_message do
    board { nil }
    sender { "" }
    reciever { "" }
    text { "MyString" }
  end
end
