class Corruption < ApplicationRecord
  belongs_to :board
  belongs_to :hero, foreign_key: :actor_id
end
