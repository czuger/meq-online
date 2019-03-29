# This module is included in board.
# To use it, call @board.

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
          raise "Board #{@board.id} : hero.playing_order should not be nil" unless next_hero.playing_order

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
