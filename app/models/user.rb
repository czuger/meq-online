require 'securerandom'

class User < ApplicationRecord

  has_many :heroes
  has_many :saurons

  has_and_belongs_to_many :boards

  IA_USER_NAME = ['IA Alice', 'IA Bob', 'IA Marc', 'IA Luc', 'IA Ida']

  def self.create_ia_user(index)
    User.find_or_create_by!(ia_user_id: index) do |user|
      user.provider = 'ia'
      user.uid = SecureRandom.uuid
      user.name =  IA_USER_NAME[index],
      user.email = "#{IA_USER_NAME[index].camelize}@foo.bar"
    end
  end

end
