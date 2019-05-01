class Mob < ApplicationRecord
  belongs_to :board

  attr_accessor :final_attack, :final_defense

  def deal_damages( damages_amount )
    damages_amount = [ damages_amount, 0 ].max
    self.life -= damages_amount
    self.damages_taken_this_turn += damages_amount
  end

  #
  # Cards methods
  #
  def card_pic_path(card_number)
    @game_mob_cards ||= GameData::MobsCards.new
    @game_mob_cards.get_card_pic_path(attack_deck, card_number)
  end

end
