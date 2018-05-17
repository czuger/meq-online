# Place all the behaviors and hooks related to the matching controller here.
# All this logic will automatically be available in application.js.
# You can use CoffeeScript in this file: http://coffeescript.org/

coordinates= () ->
  $('#map').mousemove () ->

    offset = $('#map').offset()

    page_x = event.pageX
    page_y = event.pageY

    true_x = page_x-offset.left
    true_y = page_y-offset.top

    console.log( true_x, true_y )

$(document).on('turbolinks:load'
  ->
    if window.location.pathname == '/map_coordinates/edit'
      coordinates()
)