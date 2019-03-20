module LogsHelper

  def print_log( log )
    case log.action
      when 'move'
        movement_log( log.params )
      else
        default_log( log )
    end
  end

  def show_card(log)
    if log.params['shadow_card']
      image_tag( 'sauron/shadows/' + log.params['shadow_card'] + '.png', class: 'log-card' )
    elsif log.params['plot_card']
      image_tag( 'sauron/plots/' + log.params['plot_card'] + '.png', class: 'log-card' )
    elsif log.params['event_card']
      image_tag( 'events/' + log.params['event_card'], class: 'log-card' )
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
    data = log.params.symbolize_keys

    data[:count] = data[:count].to_i if data[:count]

    # data[:name] = hero_name(data)
    t( '.' + log.action, data )
  end

  def hero_name( data )
    return 'Sauron' if data[:name] == 'sauron'
    name_code = data[:name]
    @heroes.get( name_code ).name
  end

end
