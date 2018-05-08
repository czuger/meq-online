# Place all the behaviors and hooks related to the matching controller here.
# All this logic will automatically be available in application.js.
# You can use CoffeeScript in this file: http://coffeescript.org/

card_selection = () ->
  $('.card').click () ->
    $('.card').removeClass('selected_card')
    card = $(this)
    card.addClass('selected_card')

    $('#selected_card').val( card.attr('card_id') )

    $('#play_card').removeClass('disabled')

card_hoovering = () ->
  $('.card').hover(
    () ->
      $(this).removeClass('card')
      $(this).addClass('zoomed-card')
    () ->
      $(this).removeClass('zoomed-card')
      $(this).addClass('card')
  )

$(document).on('turbolinks:load'
  ->
    if window.location.pathname.match( /boards\/\d+\/combats\/play_card/ )
      card_selection()

#    if window.location.pathname.match( /boards\/\d+\/combats/ )
#      card_hoovering()
)