# This module is included in Combat.
# And contains all mobs powers

module GameEngine
  module CombatMobsPowers

    def call_mob_power( mob, params )
      send( mob.code, params ) if respond_to?( mob.code, true )
    end

    private

    #
    # Cards power
    #
    def orc( params )
      if params.op_current.card_type == 'melee' && params.me_current.card_type == 'melee'
        params.op.deal_damages(1)
      end
    end

    def cave_troll( params )
      if params.me_current.card_type == 'ranged'
        params.op_current.cancelled = true
        params.op_current.save!
      end
    end

  end
end
