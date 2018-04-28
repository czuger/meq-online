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

  def card_used( log )
    data = log.params
    hero_name_code = data[:name]
    card_used = data[:card]
    card_name = @heroes_cards[hero_name_code][card_used][:name]
    image_tag "hero_cards/#{hero_name_code}/#{card_name}.jpg", class: 'log-card'
  end

end
