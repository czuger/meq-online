# This module is included in board.
# To use it, call @board.

module GameEngine
  module ActorActivation


    #
    # Activation state methode
    #
    def activate_current_hero
      transaction do
        set_heroes_activation_state( false )
        set_sauron_activation_state( false )
        set_hero_activation_state( current_hero.reload, true )
      end
    end

    def switch_to_sauron
      transaction do
        set_hero_activation_state( current_hero, false )
        set_sauron_activation_state( true )
      end
    end

    def heroes_actives?
      heroes.map{ |e| e.active }.inject(:|)
    end

    def set_hero_activation_state( hero, active= false )
      set_actor_activation_state( hero, active )
    end

    def set_actor_activation_state( actor, active= false )
      actor.active = active
      actor.save!
    end

    def set_heroes_activation_state( active= false )
      heroes.each do |h|
        h.active = active
        h.save!
      end
    end

    def set_sauron_activation_state( active= false )
      sauron.active = active
      sauron.save!
    end

    def set_all_actors_activation_state( active= false )
      set_sauron_activation_state active
      set_heroes_activation_state active
    end


  end
end
