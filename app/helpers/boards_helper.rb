module BoardsHelper

  def players_list( board )
    list = ''
    list += "Sauron (#{board.sauron.user.name}), " if board.sauron
    list += board.heroes.map{ |h| "#{h.name_code} (#{h.user.name})" }.join( ', ' )
    list
  end
end
