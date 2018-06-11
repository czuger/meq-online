set_selected_cards = () ->

  if $('.selected-card').length >= 1
    $('#validate').removeClass('disabled')

    selected_cards = _.map( $('.selected-card'), (c) -> $(c).attr( 'card_id' ) )
    $('#selected_cards').val( JSON.stringify( selected_cards ) )
  else
    $('#validate').addClass('disabled')

# Caution : don't mix unique and multiple selection system
# Result is put in a hidden field with id selected_cards
# If a button has an id called validate, it will be turned on or off regarding the fact that cards are selected

# If you want to allow multiple cards selection, add the classe selectable-card-selection-multiple
# to all cards
card_selection_selection_multiple = () ->
  $('.selectable-card-selection-multiple').click () ->

    card = $(this)
    if card.hasClass('selected-card')
      card.removeClass('selected-card')
    else
      card.addClass('selected-card')

    set_selected_cards()


# If you want to allow unique cards selection, add the classe selectable-card-selection-unique
# to all cards
card_selection_selection_unique = () ->
  $('.selectable-card-selection-unique').click () ->

    card = $(this)
    $('#selected_card').val( JSON.stringify( selected_cards ) )
    card.addClass('selected-card')

    set_selected_cards()


$(document).on('turbolinks:load'
  ->
    card_selection_selection_unique()
    card_selection_selection_multiple()
)
