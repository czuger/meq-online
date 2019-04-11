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

    end

  end
end
