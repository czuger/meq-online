# Place all the behaviors and hooks related to the matching controller here.
# All this logic will automatically be available in application.js.
# You can use CoffeeScript in this file: http://coffeescript.org/

card_selection = () ->
  $('.card').click () ->
    card = $(this)
    if card.hasClass('selected_card')
      card.removeClass('selected_card')
    else
      card.addClass('selected_card')


$(document).on('turbolinks:load'
  ->
    if window.location.pathname.match( /boards\/\d+\/heros\/\d+/ )
      card_selection()
)