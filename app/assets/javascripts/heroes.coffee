# Place all the behaviors and hooks related to the matching controller here.
# All this logic will automatically be available in application.js.
# You can use CoffeeScript in this file: http://coffeescript.org/

#card_selection = () ->
#  $('.card').click () ->
#    card = $(this)
#    if card.hasClass('selected_card')
#      card.removeClass('selected_card')
#    else
#      card.addClass('selected_card')
#
#    if $('.selected_card').length == 1
#      $('#move_hero').removeClass('disabled')
#      $('#card_used').val( $('.selected_card').first().attr( 'card_id' )  )
#    else
#      $('#move_hero').addClass('disabled')
#

#set_draw_card_count_equal_agility = () ->
#  $('#set_agility').click () ->
#    agility_value = $('#agility_value').val()
#    $('#nb_cards').val( agility_value )
#
#$(document).on('turbolinks:load'
#  ->
#    if window.location.pathname.match( /heros\/\d+/ )
##      card_selection()
#      set_draw_card_count_equal_agility()
#)