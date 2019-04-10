# This module is included in board.
# To use it, call self.

module GameEngine
  module ActorTurnManagement

    # Set the variable current_hero to the first hero to play
    def set_first_hero_to_play
      first_hero = heroes.order('playing_order ASC').first

      transaction do
        self.current_hero = first_hero
        self.save!

        self.heroes.update_all( turn_finished: false )
      end
    end

    def start_hero_second_turn(hero)
      if hero.turn == 2
        hero.turn = 1
        hero.save!

        actions_before_switch_to_sauron

        return false
      else
        hero.turn = 2
        hero.save!

        self.next_to_single_hero_draw!
      end
    end

    # Use this method when we have more than one heroes
    def finish_heroes_turn!
      self.transaction do
        unless switch_to_next_hero
          # If all heroes have played, we switch to sauron turn
          actions_before_switch_to_sauron
        else
          # Otherwise, we start the next hero turn
          self.next_to_rest_step!
        end

        self.save!
      end
    end
    
    private

    def actions_before_switch_to_sauron
      # At this place we need to :
      # - Call the automated rally step
      # - Call the automated story step
      self.next_to_plot!
      self.switch_to_sauron
      self.turn += 1

      self.save!
    end

    # Set the variable current_hero to  the next current hero
    # Return true if a hero had been activated
    # Return false if all heroes had finished their turn
    def switch_to_next_hero
      result = true

      transaction do
        self.current_hero.turn_finished = true
        self.current_hero.save!

        remaining_heroes = heroes.where( turn_finished: false )

        if remaining_heroes.count >= 1
          next_hero = remaining_heroes.order('playing_order ASC').first
          raise "Board #{self.id} : hero.playing_order should not be nil" unless next_hero.playing_order

          self.current_hero = next_hero
          self.activate_current_hero
          self.save!
        else
          result = false
        end
      end

      result
    end

  end
end
