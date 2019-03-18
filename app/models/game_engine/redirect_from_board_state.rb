module GameEngine
  class RedirectFromBoardState

    STATE_TO_PATH = { sauron_actions: :edit_sauron_action, event_step: :sauron_execute_event_card_screen }

    def self.redirect( board, actor )

      # If we have a route that correspond to the current board state, then we use it
      path = board.aasm_state + '_path'

      puts
      puts "RedirectFromBoardState looking for path : #{path}"
      puts

      if Rails.application.routes.url_helpers.respond_to?( path, actor )
        yield Rails.application.routes.url_helpers.send( path, actor )

      # Otherwise, if we have recorded a match, we use it
      elsif (path = STATE_TO_PATH[board.aasm_state.to_sym])
        yield Rails.application.routes.url_helpers.send( path.to_s + '_path', actor  )
      end
    end

  end
end