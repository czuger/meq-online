class Hero < ApplicationRecord
  serialize :life_pool
  serialize :rest_pool
  serialize :damage_pool
end
