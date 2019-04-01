module MapsHelper

  def get_token_positions

  end


  def get_sauron_action_token_position( token_key )
    root_token = token_key[0..-3]
    count = token_key[-1].to_i

    y = 1042
    if root_token == 'place_influence'
      x = 1595
    elsif root_token == 'draw_cards'
      x = 1692
    elsif root_token == 'command'
      x = 1788
    else
      raise "Unknown root token : #{root_token.inspect}"
    end

    x += (count-1)*30

    [ x, y ]
  end

end
