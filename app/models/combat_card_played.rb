class CombatCardPlayed < ApplicationRecord
  belongs_to :combat

  def call_power( phase, your_previous_card, opponent_previous_card, opponent_current_card)
    send(power, phase, your_previous_card, opponent_previous_card, opponent_current_card)
  end

  #
  # Cards power
  #
  def overdraw( phase, _, _, opponent_current_card )
    if phase == :current && !opponent_current_card.canceled && opponent_current_card.card_type == 'ranged'
      self.final_attack += 2
      self.save!
    end
  end

  def hack( phase, _, _, opponent_current_card )
    if phase == :current && !opponent_current_card.canceled && opponent_current_card.card_type == 'melee'
      self.final_attack += 1
      self.save!
    end
  end

  def attack_of_opportunity( phase, _, _, opponent_current_card )
    if phase == :current && (opponent_current_card.canceled || opponent_current_card.printed_attack == 0)
      self.final_attack += 5
      self.save!
    end
  end


end
