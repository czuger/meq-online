module HerosHelper

  def small_cards_classes
    card_class = ['small-card']
    card_class << @selectable_card_class if @selectable_card_class
    card_class.join( ' ' )
  end
end
