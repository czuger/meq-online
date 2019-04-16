class Mob < ApplicationRecord
  belongs_to :board

  #
  # Cards methods
  #
  def card_pic_path(card_number)
    @game_mob_cards ||= GameData::MobsCards.new
    @game_mob_cards.get_card_pic_path(attack_deck, card_number)
  end

end
