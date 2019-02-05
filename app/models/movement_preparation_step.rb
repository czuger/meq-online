class MovementPreparationStep < ApplicationRecord
  belongs_to :hero, foreign_key: :actor_id
end
