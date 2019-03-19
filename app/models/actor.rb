class Actor < ApplicationRecord
  belongs_to :board
  belongs_to :user

  has_many :senders, dependent: :destroy, class_name: 'BoardMessage', foreign_key: :sender_id
  has_many :recievers, dependent: :destroy, class_name: 'BoardMessage', foreign_key: :reciever_id
end
