# Place all the behaviors and hooks related to the matching controller here.
# All this logic will automatically be available in application.js.
# You can use CoffeeScript in this file: http://coffeescript.org/

filter= () ->
  $('#filter').on 'keyup', ->
    value = $(this).val().toLowerCase()
    $('#influences_table tr').filter ->
      $(this).toggle $(this).text().toLowerCase().indexOf(value) > -1
      return
    return

influence= () ->
  $('.influence-pawn-box' ).click () ->
    console.log( $(this).find('.influence-pawn' ).attr('location') )
    val = parseInt( $(this).find('.influence-value' ).html() ) + 1
    $(this).find('.influence-value' ).html( val )

$(document).on('turbolinks:load'
  ->
    if window.location.pathname.match( /influences\/\d+\/edit/ )
      filter()

    if window.location.pathname.match( /boards\/\d+\/map/ )
      influence()
)