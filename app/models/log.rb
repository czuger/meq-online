class Log < ApplicationRecord
  belongs_to :board
  belongs_to :actor, optional: true
end
