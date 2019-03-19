module BoardMessagesHelper

  def print_actor_name actor
    if actor.is_a? Sauron
      "Sauron (#{@board.sauron.user.name})"
    else
      @heroes = GameData::Heroes.new
      @heroes_hero = @heroes.get( actor.name_code )
      "#{@heroes_hero.name} (#{actor.user.name})"
    end
  end

  def print_sender_dest message
    if message.sender == @actor
      msg = 'To : ' + print_actor_name( message.reciever)
    else
      msg = 'From : ' + print_actor_name( message.sender)
    end
    msg + ', at : ' + message.updated_at.localtime.to_s
  end

  def alert_class message
    if message.sender == @actor
      'alert alert-info'
    else
      'alert alert-success'
    end
  end

end
