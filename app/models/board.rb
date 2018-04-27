class Board < ApplicationRecord
  has_many :heroes, dependent: :destroy
end
