# Place all the behaviors and hooks related to the matching controller here.
# All this logic will automatically be available in application.js.
# You can use CoffeeScript in this file: http://coffeescript.org/

alt_pressed = false
page_x = null
page_y = null

keyboard_alt_probe = () ->
  $(window).keydown (e) ->
    alt_pressed = e.altKey
    zoom_map()

  $(window).keyup (e) ->
    alt_pressed = e.altKey
    $('#zoom-area').hide()


zoom_map = () ->
  page_x = event.pageX if event.pageX
  page_y = event.pageY if event.pageY

  return unless alt_pressed

  offset = $('#map').offset()

  x_decal = 4890.0 / 1900.0
  y_decal = 3362.0 / 1306.0

  true_x = page_x - offset.left - (150.0 / x_decal)
  true_y = page_y - offset.top - (150.0 / y_decal)

  if true_y <= 0
    $('#zoom-area').hide()
  else
    $('#zoom-area').show()

  page_x -= 150
  page_y -= 150

  $('#zoom-area').css({left: page_x, top: page_y});
  $('#zoom-area').css('background-position', ((-true_x) * x_decal) + "px " + ((-true_y) * y_decal) + "px");

set_zoom_map = () ->
  keyboard_alt_probe()
  $('#map').mousemove(zoom_map)
  $('#zoom-area').mousemove(zoom_map)

filter= () ->
  $('#filter').on 'keyup', ->
    value = $(this).val().toLowerCase()
    $('#influences_table tr').filter ->
      $(this).toggle $(this).text().toLowerCase().indexOf(value) > -1
      return
    return

influence= () ->
  $('.influence-pawn-box' ).click () ->
    location = $(this).find('.influence-pawn' ).attr('location')
    val = parseInt( $(this).find('.influence-value' ).html() ) + 1
    $(this).find('.influence-value' ).html( val )

    actor_id = $('#actor_id').val()
    locations = {}
    locations[location]=val

    $.ajax "/influences/#{actor_id}",
      type: 'PATCH'
      data: { locations: locations, redirect: false }

$(document).on('turbolinks:load'
  ->
    if window.location.pathname.match( /maps\/\d+\/edit/ )
      influence()
      filter()
      set_zoom_map()
)