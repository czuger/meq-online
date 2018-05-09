# Place all the behaviors and hooks related to the matching controller here.
# All this logic will automatically be available in application.js.
# You can use CoffeeScript in this file: http://coffeescript.org/

zoom_map = () ->
  x = event.pageX - 150
  y = event.pageY - 150

  x_decal = 4890.0 / 1900.0
  y_decal = 3362.0 / 1306.0

  $('#zoom-area').css({left: x, top: y});
  $('#zoom-area').css('background-position', (-x * x_decal) + "px " + (-y * y_decal) + "px");

set_zoom_map = () ->
  $('#map').mousemove(zoom_map)
  $('#zoom-area').mousemove(zoom_map)

# Seems that bootstrap-toggle is not loaded by turbolinks
# So it is a good idea to interact with it to be sure it is loaded
$(document).on('turbolinks:load'
  ->
    if window.location.pathname.match( /boards\/\d+\/map/ )
      set_zoom_map()
)