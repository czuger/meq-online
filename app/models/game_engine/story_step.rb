# This module is included in actor_turn_management.
# To use it, call @board.

module GameEngine
  module StoryStep

    def advance_stories_markers
      self.story_marker_heroes += 2
      log( nil, 'story.advance', marker_name: :heroes, count: 2 )

      current_plots.each do |plot|
        case plot.story_type
          when 'conquest'
            self.story_marker_conquest += plot.story_advance
          when 'ring'
            self.story_marker_ring += plot.story_advance
          when 'corruption'
            self.story_marker_corruption += plot.story_advance
        end
        log( nil, 'story.advance', marker_name: plot.story_type, count: plot.story_advance )
      end

      check_game_end
    end

    def story_data
      heroes_to_final = 18 - story_marker_heroes

      sauron_highest_marker = [ story_marker_ring, story_marker_conquest, story_marker_corruption ].max
      sauron_to_final = 18 - sauron_highest_marker

      shadowfall_points = 0
      %w( story_marker_ring story_marker_conquest story_marker_corruption ).each do |sm|
        shadowfall_points += [ self.send( sm ), 10 ].min
      end
      sauron_to_shadowfall = 30 - shadowfall_points

      sauron_dominance = [ sauron_to_final, sauron_to_shadowfall ].min

      if heroes_to_final < sauron_dominance
        dominance = :heroes
      elsif heroes_to_final > sauron_dominance
        dominance = :sauron
      else
        dominance = :nobody
      end

      OpenStruct.new( heroes_to_final: heroes_to_final, sauron_to_final: sauron_to_final,
                      sauron_to_shadowfall: sauron_to_shadowfall,
                      dominance: dominance )
    end

    def check_game_end
      sd = story_data

      if sd.sauron_to_final <= 0 || sd.sauron_to_shadowfall <= 0
        if sd.heroes_to_final <= 0
          self.winner = 'Equality'
        else
          self.winner = 'Sauron'
        end
        finish_game!
      else
        if sd.heroes_to_final <= 0
          self.winner = 'Heroes'
          finish_game!
        end
      end

      self.save!
    end

  end
end
