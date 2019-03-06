module MapsHelper

  def get_sauron_action_token_position( token_key )
    root_token = token_key[0..-3]
    count = token_key[-1].to_i

    y = 1044
    if root_token == 'place_influence'
      x = 1597
    elsif root_token == 'draw_cards'
      x = 1693
    elsif root_token == 'command'
      x = 1789
    else
      raise "Unknown root token : #{root_token.inspect}"
    end

    x += (count-1)*30

    [ x, y ]
  end

end
