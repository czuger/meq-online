class Player < ApplicationRecord
  belongs_to :board
  belongs_to :user
end
