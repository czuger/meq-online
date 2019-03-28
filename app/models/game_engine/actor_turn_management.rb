# This module is included in board.
# To use it, call @board.

module GameEngine
  module ActorTurnManagement

    # Set the variable current_hero to  the next current hero
    # Return true if a hero had been activated
    # Return false if all heroes had finished their turn
    def switch_to_next_hero
      remaining_heroes = heroes.where( turn_finished: false )

      return false if remaining_heroes.count == 0

      next_hero = remaining_heroes.order('playing_order ASC').first

      raise "Board #{@board.id} : hero.playing_order should not be nil" unless next_hero.playing_order

      transaction do
        self.current_hero = next_hero
        self.switch_to_current_hero
        self.save!
      end

      true
    end

  end
end
