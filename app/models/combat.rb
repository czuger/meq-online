class Combat < ApplicationRecord
  include AASM

  belongs_to :board
  belongs_to :hero, class_name: 'Actor', foreign_key: :actor_id
  belongs_to :mob

  # serialize :hero_cards_played
  # serialize :sauron_cards_played
  # serialize :sauron_hand

  # aasm do
  #   state :hero_choices, :initial => true
  #   state :started
  #
  #   event :start do
  #     transitions :from => :hero_choices, :to => :started
  #   end
  # end

  def play_card( board, sauron, player, card_to_play )
    transaction do
      if sauron
        update( sauron_card_to_play: card_to_play )
        # DONT LOG THIS IT IS SECRET
        # board.logs.create!( action: 'combat.play_card', params: { name: 'sauron' }, card_pic_path: 'monsters/' + player.cards[card_to_play.to_i].pic_path )
      else
        update( hero_card_to_play: card_to_play )
        # board.logs.create!( action: 'combat.play_card', params: { name: hero.code_name, card: card_to_play } )
      end
    end
  end

end
