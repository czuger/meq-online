module GameEngine
  class RedirectFromBoardState

    STATE_TO_PATH = {}

    def self.redirect( board, actor )

      # If we have a route that correspond to the current board state, then we use it
      path = board.aasm_state + '_path'
      if Rails.application.routes.url_helpers.respond_to?( path, actor )
        yield Rails.application.routes.url_helpers.send( path, actor )

      # Otherwise, if we have recorded a match, we use it
      # elsif STATE_TO_PATH[board.aasm_state]
      #   yield Rails.application.routes.url_helpers.send( STATE_TO_PATH[board.aasm_state] + '_path', actor  )
      end
    end

    # def self.return_path( board, actor )
    #
    #   # If we have a route that correspond to the current board state, then we use it
    #   path = board.aasm_state + 'return_path'
    #   if Rails.application.routes.url_helpers.respond_to?( path, actor )
    #     yield Rails.application.routes.url_helpers.send( path, actor )
    #
    #     # Otherwise, if we have recorded a match, we use it
    #   # elsif STATE_TO_PATH[board.aasm_state]
    #   #   yield Rails.application.routes.url_helpers.send( STATE_TO_PATH[board.aasm_state] + '_path', actor  )
    #   end
    # end

  end
end