class CombatCardPlayed < ApplicationRecord
  belongs_to :combat

  include GameEngine::CombatCardPowers

  def call_power( phase, combat_params )
    unless cancelled
      @combat_params = combat_params
      @phase = phase
      send(power)
    end
  end

  private

  #
  # Phases methods
  #
  def current?
    @phase == :current
  end

  def current_cancel?
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
    @combat_params.op_current.card_type == 'melee'.freeze
  end

  def op_current_ranged?
    @combat_params.op_current.card_type == 'ranged'.freeze
  end

  def me_current_ranged?
    @combat_params.me_current.card_type == 'ranged'.freeze
  end

  #
  # Cancellation methods
  #
  def op_current_cancelled?
    @combat_params.op_current.cancelled
  end

  def op_current_cancel!
    @combat_params.op_current.final_attack = 0
    @combat_params.op_current.final_defense = 0
    @combat_params.op_current.cancelled = true
    @combat_params.op_current.save!
  end

  def cancellation_dont_break( *args )
    return false if cancelled
    args.each do |a|
      return false if @combat_params.send(a)&.cancelled
    end
    true
  end
end
