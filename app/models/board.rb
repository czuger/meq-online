class Board < ApplicationRecord

  include GameEngine::BoardAasm
  include GameEngine::ActorActivation

  has_many :logs, dependent: :destroy

  has_many :heroes, dependent: :destroy
  has_one :sauron, dependent: :destroy

  has_and_belongs_to_many :users

  belongs_to :current_hero, class_name: 'Hero', optional: true

  #
  # Log methodes
  #
  def log( actor, action, params= {} )
    logs.create!( board: self, actor: actor, action: action, params: params )
  end

end
