module BoardsHelper

  def players_list( board )
    list = []
    list << "Sauron (#{board.sauron.user.name})" if board.sauron
    list += board.heroes.map{ |h| "#{h.name_code} (#{h.user.name})" }
    list.join( ', ' )
  end

  def hero_link( board, hero_name_code )
    heroes_hash = Hash[ board.heroes.select{ |h| h.user_id == current_user.id }.map{ |h| [ h.name_code.to_sym, h ] } ]

    link_to hero_name_code.capitalize, GameEngine::RouteFromBoardState.get_route( board,heroes_hash[ hero_name_code ] ) if heroes_hash[ hero_name_code ]
  end

  def sauron_link( board, role: nil, klass: nil )
    if board.sauron&.user_id == current_user.id
      link_to 'Sauron', GameEngine::RouteFromBoardState.get_route( board,board.sauron ), role: role, class: klass
    end
  end


end
