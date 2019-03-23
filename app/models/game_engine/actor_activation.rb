module GameEngine
  module ActorActivation

    #
    # Activation state methode
    #

    def activate_current_hero
      transaction do
        set_hero_activation_state( current_hero, true )
        set_sauron_activation_state( false )
      end
    end

    def heroes_actives?
      heroes.map{ |e| e.active }.inject(:|)
    end

    def set_hero_activation_state( hero, active= false )
      hero.active = active
      hero.save!
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
