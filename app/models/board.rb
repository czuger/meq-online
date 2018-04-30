class Board < ApplicationRecord
  has_many :heroes, dependent: :destroy
  has_one :sauron, dependent: :destroy
  has_many :logs, dependent: :destroy
end
