class BoardMessage < ApplicationRecord
  belongs_to :reciever, class_name: 'Actor'
  belongs_to :sender, class_name: 'Actor'
end
