module GameEngine
  class InfluencePaths

    def initialize( board )
      @board = board
    end

    def max_for_new_token( location )
      get_max_influence_for_location( location ) || 0
    end

    def max_for_existing_token( location )
      get_max_influence_for_location( location ) || 1
    end

    private

    def get_max_influence_for_location( location )
      get_connected_shadow_strongholds( location ).map{ |stronghold| @board.influence[stronghold] }.max
    end

    def get_connected_shadow_strongholds( location )
      connected_locations = []
      %w( dol_guldur barad_dur mount_gundabad ).each do |stronghold|
        result, _ = find( location, stronghold )
        connected_locations << stronghold if result
      end
      connected_locations
    end

    # Call with : find( map, movement_graph, [], [ location.q, location.r ], location, n )
    def find( location, destination )

      game_data_locations_path = GameData::LocationsPaths.new
      
      frontier = Containers::PriorityQueue.new()
      frontier.push(location, 0)
      frontier_history = []
      came_from = {}
      cost_so_far = {}
      came_from[ location ] = nil
      cost_so_far[ location ] = 0
      path_found = false

      while not frontier.empty?
        current = frontier.pop()
        frontier_history << current unless frontier_history.include?( current )

        if current == destination
          path_found = true
          break
        end

        game_data_locations_path.get_connected_locations( current ).each do |next_location|
          # p next_location

          new_cost = movement_cost( next_location )
          if new_cost
            new_cost = cost_so_far[ current ] + movement_cost( next_location )
          else
            next
          end

          if ( not cost_so_far[ next_location ] ) or new_cost < cost_so_far[ next_location ]
            cost_so_far[ next_location ] = new_cost
            priority = new_cost
            frontier.push( next_location, priority )
            came_from[ next_location ] = current
          end
        end
      end

      [path_found, frontier_history]
    end

    def movement_cost( next_location )
      return 1 if @board.influence[next_location].to_i >= 1
      nil
    end
  end
end
