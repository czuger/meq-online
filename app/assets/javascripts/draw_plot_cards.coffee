# Place all the behaviors and hooks related to the matching controller here.
# All this logic will automatically be available in application.js.
# You can use CoffeeScript in this file: http://coffeescript.org/

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

#card_hoovering = () ->
#  $('.card').hover(
#    () ->
#      $(this).removeClass('card')
#      $(this).addClass('zoomed-card')
#    () ->
#      $(this).removeClass('zoomed-card')
#      $(this).addClass('card')
#  )

$(document).on('turbolinks:load'
  ->
    if window.location.pathname.match( /draw_plot_cards\/\d+\/edit/ )
      card_selection()
)
