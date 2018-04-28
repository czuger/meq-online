class Log < ApplicationRecord
  belongs_to :board
  serialize :params
end
