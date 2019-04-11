module HerosHelper

  def small_cards_classes
    card_class = ['small-card']
    card_class << @selectable_card_class if @selectable_card_class
    card_class.join( ' ' )
  end

  def get_token_text(token)
    case token.type
      when :favor
        [true, t('.explore.favor')]
      when :character
        [true, t('.explore.character', character_name: token.name)]
      when :plot
        plot = @board.current_plots.where(affected_location: @actor.location).first

        if plot.favor_to_discard <= @actor.favor
          [true, t('.explore.plot.can_discard')]
        else
          [false, t('.explore.plot.cannot_discard')]
        end

      else
        nil
    end
  end

end