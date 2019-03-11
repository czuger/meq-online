module PlotCardsHelper

  def draw_screen_path
    Rails.application.routes.url_helpers.send( "#{controller_to_code}_cards_draw_path", @actor )
  end

  def draw_screen_title_name
    "Choose numbers of #{controller_to_code} cards to draw"
  end

  def keep_screen_path
    Rails.application.routes.url_helpers.send( "#{controller_to_code}_cards_keep_path", @actor )
  end

  def keep_screen_title_name
    "Select #{controller_to_code}(s) card(s) to keep"
  end

  def card_picture( card_number, selectable: :none )
    image_class = %w( medium-card )
    image_class << "selectable-card-selection-#{selectable}" if selectable != :none

    image_tag "sauron/#{controller_to_code}s/#{card_number}.png", class: image_class, card_id: card_number
  end

  private

  # We get only the prefix of the controller
  # To use this, we must name the controller like action_cards
  # Example : corruption_cards, event_cards, encounter_cards etc ...
  def controller_to_code
    controller_name[0..-7]
  end

end
