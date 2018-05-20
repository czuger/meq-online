module BoardsHelper

  def players_list( board )
    list = []
    list << "Sauron (#{board.sauron.user.name})" if board.sauron
    list += board.heroes.map{ |h| "#{h.name_code} (#{h.user.name})" }
    list.join( ', ' )
  end

  def hero_link( board, hero_name_code )
    heroes_hash = Hash[ board.heroes.select{ |h| h.user_id == current_user.id }.map{ |h| [ h.name_code.to_sym, h ] } ]
    p heroes_hash
    link_to @heroes.get( hero_name_code ).name , heroes_hash[ hero_name_code ] if heroes_hash[ hero_name_code ]
  end


end
