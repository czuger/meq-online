class User < ApplicationRecord

  has_many :heroes
  has_many :saurons

  has_and_belongs_to_many :boards

end
