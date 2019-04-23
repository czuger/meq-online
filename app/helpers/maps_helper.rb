module MapsHelper

  def shadow_pool_tokens
    y_position = 1183*@mul_y
    pool_token_number = 0
    1.upto( 12 ).to_a.in_groups_of(4).each do |row|
      x_position = 1695*@mul_x
      row.each do |element|
        if pool_token_number < @board.shadow_pool
          value = 1
        else
          value = 0
        end
        yield(y_position, x_position, pool_token_number, value)
        pool_token_number += 1
        x_position += 38.5*@mul_x
      end
      y_position += 35*@mul_y
    end
  end

  def get_token_positions
    @tokens.each do |loc, tokens|
      loc = @locations.get( loc )

      tokens_count = tokens.count
      first_token = tokens.sort_by { |t| t.priority }.first

      tokens_hover_text = tokens.map{ |t| monster_name_sauron_filter(t) }.join( ', ' )

      px = loc.pos_x*@mul_x-23-15
      py = loc.pos_y*@mul_y-25-16

      token_decal = first_token.type == :plot ? 7 : 0

      py += token_decal

      pic_path = first_token.pic_path

      if @actor.is_a?(Sauron) && first_token.sauron_pic_path
        pic_path = first_token.sauron_pic_path
      end

      token_pic_class = 'token-on-map-pic'
      case first_token.type
        when :minion
          token_pic_class = 'token-on-map-pic-minion'
        when :character
          px += 4
          py += 4
        when :favor
          token_pic_class = 'token-on-map-pic-favor'
          px += 9
          py += 9
        when :plot
          px += 7
          py += 12
        else
          px += 4
          py += 4
      end

      style = "top:#{py}px; left:#{px}px;"

      yield pic_path, style, tokens_hover_text, tokens_count, token_decal, token_pic_class
    end
  end


  def get_sauron_action_token_position( token_key )
    root_token = token_key[0..-3]
    count = token_key[-1].to_i

    y = 1614
    if root_token == 'place_influence'
      x = 2463
    elsif root_token == 'draw_cards'
      x = 2612
    elsif root_token == 'command'
      x = 2761
    else
      raise "Unknown root token : #{root_token.inspect}"
    end

    x += (count-1)*46

    [ x, y ]
  end

  private

  # This method is used to filter the monsters names (Monster for heroes map and real monster name for Sauron)
  def monster_name_sauron_filter( token_data )
    if @actor.is_a?(Sauron) && token_data.sauron_name
      token_data.sauron_name
    else
      token_data.name
    end
  end

end
