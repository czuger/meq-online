module LogsHelper

  def print_log( log )
    case log.action
      when 'move'
        movement_log( log.params )
      else
        default_log( log )
    end
  end

  def movement_log( data )
    t( '.move',
      name: hero_name( data ),
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

  def default_log( log )
    data = log.params
    # data[:name] = hero_name(data)
    t( '.' + log.action, data )
  end

  def hero_name( data )
    return 'Sauron' if data[:name] == 'sauron'
    name_code = data[:name]
    @heroes.get( name_code ).name
  end

end
