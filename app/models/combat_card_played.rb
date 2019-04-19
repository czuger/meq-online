class CombatCardPlayed < ApplicationRecord
  belongs_to :combat

  def call_power( phase, combat_params )
    @combat_params = combat_params
    @phase = phase
    send(power)
  end

  private

  #
  # Cards power
  #
  def smash
    if current? && cancellation_dont_break()
      @combat_params.op_current.final_defense -= @combat_params.op_current.printed_defense
      @combat_params.op_current.printed_defense = 0

      @combat_params.op_current.save!
    end
  end

  def reckless 
    if after? && cancellation_dont_break
      if @combat_params.me.damages_taken_this_turn >= 1
        @combat_params.me.take_damages(2)
      end
    end
  end

  def charge 
    if current? && cancellation_dont_break( :op_current ) && op_current_ranged?
      op_current_cancel!
    end
  end

  def parry 
    if current? && cancellation_dont_break( :op_current ) && op_current_melee?
      self.final_defense += 2
      self.save!
    end
  end

  def evade
    if current? && cancellation_dont_break( :op_current ) && op_current_melee?
      op_current_cancel!
    end
  end

  def fall_back 
    if previous? && cancellation_dont_break( :op_current ) && op_current_melee?
      op_current_cancel!
    end
  end

  def aimed_shot 
    if current? && cancellation_dont_break( :op_current, op_previous ) &&
        opponent_current_card.card_type == opponent_previous_card.card_type
      self.final_attack += 3
      self.save!
    end
  end

  def ranged_strike
    if current? && cancellation_dont_break( :op_current ) && op_current_ranged?
      self.final_attack += 2
      self.save!
    end
  end

  def hack 
    if current? && cancellation_dont_break( :op_current ) && op_current_melee?
      self.final_attack += 1
      self.save!
    end
  end

  def attack_of_opportunity 
    if current? && cancellation_dont_break && (op_current_canceled? || opponent_current_card.printed_attack == 0)
      self.final_attack += 5
      self.save!
    end
  end

  #
  # Phases methods
  #
  def current?
    @phase == :current
  end

  def previous?
    @phase == :previous
  end

  def after?
    @phase == :after
  end

  #
  # Cards status methods
  #
  def op_current_melee?
    @combat_params.op_current.card_type == :melee
  end

  def op_current_ranged?
    @combat_params.op_current.card_type == :ranged
  end

  #
  # Cancellation methods
  #
  def op_current_cancelled?
    @combat_params.op_current.cancelled
  end

  def op_current_cancel!
    @combat_params.op_current.cancel!
  end

  def cancel!
    self.final_attack = 0
    self.final_defense = 0
    self.cancelled = true
    self.save!
  end

  def cancellation_dont_break( *args )
    return true if cancelled
    args.each do |a|
      return true if @combat_params.send(a).cancelled
    end
    false
  end
end
