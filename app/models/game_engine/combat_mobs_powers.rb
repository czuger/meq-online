# This module is included in Combat.
# And contains all mobs powers

module GameEngine
  module CombatMobsPowers

    def call_mob_power( mob, params )
      send( mob.code, params ) if respond_to?( mob.code )
    end

    private

    #
    # Cards power
    #
    def orc( params )
      if params.op_current_card.card_type == 'melee' && params.me_current_card.card_type == 'melee'
        params.op.deal_damages( 1 )
      end
    end

  end
end
