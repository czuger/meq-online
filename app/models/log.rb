class Log < ApplicationRecord
  belongs_to :board
  belongs_to :user
  belongs_to :actor

  serialize :params
end
