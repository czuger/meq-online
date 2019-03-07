set_selected_cards_multiple = () ->

  if $('.selected-card').length >= 1
    $('#validate_multiple').removeAttr('disabled')

    selected_cards = _.map( $('.selected-card'), (c) -> $(c).attr( 'card_id' ) )
    $('#selected_cards').val( selected_cards )
  else
    $('#validate_multiple').attr('disabled', 'disabled')


# If you want to allow multiple cards selection, add the classe selectable-card-selection-multiple
# to all cards
card_selection_selection_multiple = () ->
  $('.selectable-card-selection-multiple').click () ->

    card = $(this)
    console.log( 'card=', card, card.hasClass('selected-card'))
    if card.hasClass('selected-card')
      card.removeClass('selected-card')
    else
      card.addClass('selected-card')

    set_selected_cards_multiple()


# If you want to allow unique cards selection, add the classe selectable-card-selection-unique
# to all cards
card_selection_selection_unique = () ->
  $('.selectable-card-selection-unique').click () ->

    card = $(this)
    $(".selected-card").removeClass("selected-card")
    card.addClass('selected-card')

    $('#selected_card').val( card.attr('card_id') )

    $('#validate_unique').removeAttr('disabled')

$(document).on('turbolinks:load'
  ->
    card_selection_selection_unique()
    card_selection_selection_multiple()
)
