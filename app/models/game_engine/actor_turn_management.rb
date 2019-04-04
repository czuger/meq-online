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

    def finish_hero_turn!
      self.transaction do

      unless switch_to_next_hero
        # If all heroes have played, we switch to sauron turn

        # At this place we need to :
        # - Call the automated rally step
        # - Call the automated story step
        self.next_to_plot!
        self.switch_to_sauron
      else
        # Otherwise, we switch to the sauron ability to play a shadow card at the beginning of the player
        self.next_to_play_shadow_card_at_start_of_hero_turn!
        self.switch_to_sauron
      end

      self.save!
      end
    end
    
    private

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
