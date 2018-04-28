module LogsHelper

  def print_log( log )
    case when log.action == 'move'
        movement_log( log.params )
    end
  end

  def movement_log( data )
    t( '.move',
      name: @heroes[data[:name]][:name],
      from: @locations[data[:from]][:name],
      to: @locations[data[:to]][:name],
      card: @heroes_cards[data[:name]][data[:card]][:name]
    )
  end

end
