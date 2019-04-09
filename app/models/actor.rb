class Actor < ApplicationRecord
  belongs_to :board
  belongs_to :user

  has_many :senders, dependent: :destroy, class_name: 'BoardMessage', foreign_key: :sender_id
  has_many :recievers, dependent: :destroy, class_name: 'BoardMessage', foreign_key: :reciever_id

  def assert_sauron
    raise "#{self.inspect} should be a Sauron type." unless self.is_a? Sauron
  end

end
