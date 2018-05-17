# Place all the behaviors and hooks related to the matching controller here.
# All this logic will automatically be available in application.js.
# You can use CoffeeScript in this file: http://coffeescript.org/

zoom_map = () ->

  offset = $('#map').offset()

  page_x = event.pageX
  page_y = event.pageY

  x_decal = 4890.0 / 1900.0
  y_decal = 3362.0 / 1306.0

  true_x = page_x-offset.left-(150.0/x_decal)
  true_y = page_y-offset.top-(150.0/y_decal)

  if true_y <= 0
    $('#zoom-area').hide()
  else
    $('#zoom-area').show()

  page_x-=150
  page_y-=150

  $('#zoom-area').css({left: page_x, top: page_y});
  $('#zoom-area').css('background-position', ((-true_x) * x_decal) + "px " + ((-true_y) * y_decal) + "px");

set_zoom_map = () ->
  $('#map').mousemove(zoom_map)
  $('#zoom-area').mousemove(zoom_map)

# Seems that bootstrap-toggle is not loaded by turbolinks
# So it is a good idea to interact with it to be sure it is loaded
$(document).on('turbolinks:load'
  ->
#    if window.location.pathname.match( /boards\/\d+\/map/ )
#      set_zoom_map()
)