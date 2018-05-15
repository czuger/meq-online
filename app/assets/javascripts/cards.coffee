selected_cards = []

card_selection = () ->
  $('.selectable-card').click () ->
    card = $(this)
    if card.hasClass('selected-card')
      card.removeClass('selected-card')
    else
      card.addClass('selected-card')

    if $('.selected-card').length >= 1
      $('#validate').removeClass('disabled')

      selected_cards = _.map( $('.selected-card'), (c) -> $(c).attr( 'card_id' ) )
      $('#selected_card').val( JSON.stringify( selected_cards ) )
    else
      $('#validate').addClass('disabled')

$(document).on('turbolinks:load'
  ->
    card_selection()
)
