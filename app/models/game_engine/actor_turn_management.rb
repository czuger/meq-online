# This module is included in board.
# To use it, call self.

module GameEngine
  module ActorTurnManagement

    include StoryStep

    # Set the variable current_hero to the first hero to play
    def set_first_hero_to_play
      first_hero = heroes.order('playing_order ASC').first

      transaction do
        self.current_hero = first_hero
        self.save!

        activate_current_hero

        self.heroes.update_all( turn_finished: false )
      end
    end

    def start_hero_second_turn(hero)
      if hero.turn == 2
        hero.turn = 1
        hero.save!

        # Hero turn is finished in this case
        actions_before_switch_to_sauron

        return false
      else
        hero.turn = 2
        hero.save!

        activate_current_hero

        self.next_to_finish_hero_turn!
        self.next_to_hero_draw_cards_screen!
      end
    end

    # Use this method when we have more than one heroes
    def finish_heroes_turn!(hero)

      controller_redirect_to = :boards

      self.transaction do
        hero_end_turn_operations(hero)

        # If we have more than one player
        if current_heroes_count > 1

          unless switch_to_next_hero
            # If all heroes have played, we switch to sauron turn
            actions_before_switch_to_sauron
          end
        else
          # If we have only one player
          if start_hero_second_turn(hero)
            # We started a new turn for hero
            controller_redirect_to = :hero_draw_cards_screen
          end
        end

        self.save!
        controller_redirect_to
      end
    end

    def hero_end_turn_operations(actor)
      actor.used_powers['argalad'] = false
      actor.save!
    end

    private

    def actions_before_switch_to_sauron
      rally_step

      self.next_to_finish_hero_turn!

      old_stage = self.story_stage
      advance_stories_markers

      new_stage = self.story_stage
      if new_stage > old_stage
        if new_stage == 2
          create_monster( :gothmog, :mount_gundabad )
          create_monster( :ringwraiths, :minas_morgul )
        end
        if new_stage == 3
          create_monster( :witch_king, :minas_morgul )
        end
      end

      unless self.finished?
        self.next_to_play_screen_sauron_plot_cards!
        self.switch_to_sauron
        self.turn += 1

        self.save!
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
          raise "Board #{self.id} : hero.playing_order should not be nil" unless next_hero.playing_order

          self.current_hero = next_hero
          self.save!

          self.activate_current_hero

          self.next_to_finish_hero_turn!
          self.next_to_hero_draw_cards_screen!
        else
          result = false
        end
      end

      result
    end

    def rally_step
      heroes.each do |hero|
        local_influence = influence[hero.location].to_i
        if local_influence > 0
          log(nil, 'rally.hero_remove', hero: hero.name, location: location_name(hero.location), count: local_influence )
          influence.delete(hero.location)
        end
      end
      i_path = GameEngine::InfluencePaths.new(self)
      influence.keys.each do |location|
        location_max = i_path.max_for_existing_token(location)
        if location_max < influence[location]
          log(nil, 'rally.auto_remove', location: location_name(location), count: influence[location] - location_max )
          influence[location] = location_max
        end
      end
    end

  end
end
