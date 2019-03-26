module GameEngine
  class RouteFromBoardState

    STATE_TO_PATH = {
      sauron_actions: :edit_sauron_action,
      event_step: :edit_event,
      heroes_draw_cards: :hero_draw_cards_screen,
      play_shadow_card_at_start_of_hero_turn: :start_hero_turn_play_card_screen_sauron_shadow_cards,
      rest_step: :hero_rest_screen,
      movement_preparation_step: :hero_movement_preparation_steps,
    }

    def self.get_route( board, actor )

      if actor.active
        path = board.aasm_state + '_path'

        # If we have a method for that state (this is useful when heroes and Sauron are active)
        if self.respond_to?( board.aasm_state )
          self.send( board.aasm_state, board, actor )

        # Otherwise, if we have a route that correspond to the current board state, then we use it
        elsif Rails.application.routes.url_helpers.respond_to?( path, actor )
          Rails.application.routes.url_helpers.send( path, actor )

        # Otherwise, if we have recorded a match, we use it
        elsif (path = STATE_TO_PATH[board.aasm_state.to_sym])
          Rails.application.routes.url_helpers.send( path.to_s + '_path', actor  )

        else
          raise "Route not found for state #{board.aasm_state}"
        end

      else
        Rails.application.routes.url_helpers.send( 'board_inactive_actor_path', board )
      end
    end

    # def self.event_state( board, actor )
    #   if actor.is_a?
    # end

  end
end
