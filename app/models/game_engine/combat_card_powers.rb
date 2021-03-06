# This module is included in CombatCardPlayer.
# And contains all cards powers

module GameEngine
  module CombatCardPowers

    private

    #
    # Cards power
    #
    def precision
      smash
    end

    def smash
      if current? && cancellation_dont_break()
        @combat_params.op_current.printed_defense = 0

        @combat_params.op_current.save!
      end
    end

    def deadly_finesse
      if current? && cancellation_dont_break()
        @combat_params.op.final_defense -= (@combat_params.op_current.printed_defense - 1)
        @combat_params.op.final_attack -= (@combat_params.op_current.printed_attack - 1)
        @combat_params.op_current.printed_defense = 1
        @combat_params.op_current.printed_attack = 1

        @combat_params.op_current.save!
      end
    end

    def reckless
      if after? && cancellation_dont_break
        if @combat_params.me.damages_taken_this_turn >= 1
          @combat_params.me.deal_damages(2)
        end
      end
    end

    def charge
      if current_cancel? && cancellation_dont_break( :op_current ) && op_current_ranged?
        op_current_cancel!
      end
    end

    def parry
      if current? && cancellation_dont_break( :op_current ) && op_current_melee?
        @combat_params.me.final_defense += 2
      end
    end

    def evade
      if current_cancel? && cancellation_dont_break( :op_current ) && op_current_melee?
        op_current_cancel!
      end
    end

    def concentrate
      if previous? && cancellation_dont_break( :me_previous )
        @combat_params.me.final_attack += 2
        @combat_params.me.final_defense += 2
      end
    end

    def fall_back
      if previous? && cancellation_dont_break( :me_previous, :op_current ) && op_current_melee?
        op_current_cancel!
      end
    end

    def volley
      if previous? && cancellation_dont_break( :me_previous, :me_current ) && me_current_ranged?
        @combat_params.me.final_attack += 2
      end
    end

    def aimed_shot
      if current? && cancellation_dont_break( :op_current, :op_previous ) &&
          @combat_params.op_current.card_type == @combat_params.op_previous&.card_type
        @combat_params.me.final_attack += 3
      end
    end

    def overdraw
      ranged_strike
    end

    def ranged_strike
      if current? && cancellation_dont_break( :op_current ) && op_current_ranged?
        @combat_params.me.final_attack += 2
      end
    end

    def hack
      if current? && cancellation_dont_break( :op_current ) && op_current_melee?
        @combat_params.me.final_attack += 1
      end
    end

    def attack_of_opportunity
      if current? && cancellation_dont_break && (op_current_cancelled? || @combat_params.op_current.printed_defense == 0)
        @combat_params.me.final_attack += 5
      end
    end


  end
end
