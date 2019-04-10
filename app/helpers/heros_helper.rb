module HerosHelper

  def small_cards_classes
    card_class = ['small-card']
    card_class << @selectable_card_class if @selectable_card_class
    card_class.join( ' ' )
  end

  def get_token_text(token)
    case token.type
      when :favor
        t('.explore.favor')
      when :character
        t('.explore.character', character_name: token.name)
      else
        nil
    end
  end

end