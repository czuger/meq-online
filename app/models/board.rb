class Board < ApplicationRecord
  has_many :heroes, dependent: :destroy
  has_one :sauron, dependent: :destroy
  has_many :logs, dependent: :destroy

  has_and_belongs_to_many :users

end
