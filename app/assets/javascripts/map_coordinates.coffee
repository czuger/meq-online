# Place all the behaviors and hooks related to the matching controller here.
# All this logic will automatically be available in application.js.
# You can use CoffeeScript in this file: http://coffeescript.org/

coordinates= () ->
  $('#map').click () ->

    offset = $('#map').offset()

    page_x = event.pageX
    page_y = event.pageY

    true_x = page_x-offset.left
    true_y = page_y-offset.top

    console.log( true_x, true_y )

    $.post '/map_coordinates/update', true_x: true_x, true_y: true_y, current_place_code: $('#current_place_code').val()

$(document).on('turbolinks:load'
  ->
    if window.location.pathname == '/map_coordinates/edit'
      coordinates()
      alert( "Click in the center of #{$('#current_place_name').val()}")

      window.scrollTo( parseInt( $('#scroll_x').val()-800 ), parseInt( $('#scroll_y').val()-400 ) );
)