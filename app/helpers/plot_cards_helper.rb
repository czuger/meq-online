module PlotCardsHelper

  def draw_screen_path
    Rails.application.routes.url_helpers.send( "#{controller_to_code}_cards_draw_path", @actor )
  end

  def title_name
    "Choose numbers of #{controller_to_code} cards to draw"
  end

  def card_picture( card_number )
    image_tag "sauron/#{controller_to_code}/#{card_number}.png", class: 'medium-card'
  end

  private

  # We get only the prefix of the controller
  # To use this, we must name the controller like action_cards
  # Example : corruption_cards, event_cards, encounter_cards etc ...
  def controller_to_code
    controller_name[0..-7]
  end

end
