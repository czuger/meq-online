module GameEngine
  class RouteFromBoardState

    def initialize
      @computed_routes = Hash[Rails.application.routes.routes.map { |r| [r.name, r.path.spec.to_s] }]
    end

    def get_route( board, actor )
      found_route = get_screen_route(board)
      if found_route
        return create_route(found_route, board, actor)
      end

      raise "Unable to fin route for #{board.aasm_state}"
    end

    private

    def create_route(route, board, actor)

      params = get_route_params_hash(route, board, actor)
      route += '_path'

      if Rails.application.routes.url_helpers.respond_to?( route, params )
        return Rails.application.routes.url_helpers.send( route, params.values )
      else
        raise "Route #{route} does not match with params : #{params.keys}"
      end
    end

    def sauron_screen_path(board)
      'sauron_' + board.aasm_state + 'screen_path'
    end

    def get_screen_route(board)
      @computed_routes.keys.each do |route_key|
        return route_key if route_key =~ /#{board.aasm_state + '_screen'}/
      end
      nil
    end

    def get_route_params_hash(route, board, actor)
      params_to_check = %w( board_id actor_id sauron_id hero_id )
      params = params_to_check.select{ |p| @computed_routes[route] =~ /#{p}/ }

      Hash[params.map{ |p| [ convert_param(p), param_to_variable(p, board, actor) ] }]
    end

    def convert_param(param)
      param = 'actor_id' if param == 'sauron_id' || param == 'hero_id'
      param
    end

    # This transform the param name (actor_id to a bind to the local actor variable)
    def param_to_variable(param_name, board, actor)
      binding.local_variable_get( convert_param(param_name).gsub( '_id', '' ) )
    end
  end
end
