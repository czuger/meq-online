# Place all the behaviors and hooks related to the matching controller here.
# All this logic will automatically be available in application.js.
# You can use CoffeeScript in this file: http://coffeescript.org/

alt_pressed = false
shift_pressed = false
page_x = null
page_y = null

keyboard_probe = () ->
  $(window).keydown (e) ->
    alt_pressed = e.altKey
    shift_pressed = e.shiftKey
    zoom_map()

  $(window).keyup (e) ->
    alt_pressed = e.altKey
    shift_pressed = e.shiftKey
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
  keyboard_probe()
  $('#map').mousemove(zoom_map)
  $('#zoom-area').mousemove(zoom_map)

influence= () ->
  $('.influence-pawn-box' ).click () ->
    location = $(this).find('.influence-pawn' ).attr('location')

    val = parseInt( $(this).find('.influence-value' ).html() )

    if shift_pressed
      val -= 1 if val > 0
    else
      val += 1

    actor_id = $('#actor_id').val()
    locations = {}
    locations[location]=val

    $.post "/sauron/#{actor_id}/sauron_actions/set_influence",
      location: location
      val: val
      (data) ->

        if data.result
          $("#influence-value-#{data.location}").html( data.val )
        else
#          console.log( data )
          p = $("#influence-value-#{data.location}").parent().position()

#          console.log(p)

          alert = $( '#influence-alert' )
          unless alert.is(":visible")
            alert.html(data.message)
            alert.html(data.message)
            alert.css('top', p.top-80 + "px");
            alert.css('left', p.left-180 + "px");
            alert.show()
            setTimeout(hide_influence_alert, 2000);

hide_influence_alert= () ->
  $( '#influence-alert' ).fadeOut()

zoom_plot_cards= () ->
  $('.plot-card-on-map').hover () ->
    console.log($(this))
    $(this).addClass('zoomed-plot-card')
  , ->
    console.log($(this))
    $(this).removeClass('zoomed-plot-card')

#
# Shadow pool tokens
#
shadow_pool= () ->
  $('.influence-pool-value' ).click () ->

    current_val = parseInt( $(this).html() )
    actor_id = $('#actor_id').val()

    $.ajax "/shadow_pools/#{actor_id}/update_from_map",
      type: 'PATCH'
      data: { current_val: current_val }
      success: (result) ->
        console.log( result )
        for i in [ 0 .. result ]
#          console.log( i )
          value = 1 if i < result
          value = 0 if i >= result
          $("#pool_token_id_#{i}").html( value )


$(document).on('turbolinks:load'
  ->
    if window.location.pathname.match( /maps\/\d+\/edit/ ) || window.location.pathname.match( /maps\/\d+/ )
#      set_zoom_map()
      zoom_plot_cards()

    if window.location.pathname.match( /maps\/\d+\/edit/ )
      keyboard_probe()
      influence()
      shadow_pool()
)