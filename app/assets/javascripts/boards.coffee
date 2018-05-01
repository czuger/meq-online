# Place all the behaviors and hooks related to the matching controller here.
# All this logic will automatically be available in application.js.
# You can use CoffeeScript in this file: http://coffeescript.org/

# See : http://www.bootstraptoggle.com/
sauron_checkbox = () ->
  $('#sauron').bootstrapToggle({
    on: 'Yes',
    off: 'No'
  })


match_players_count = () ->
  $('.select-players').change () ->
    players_count = parseInt( $(this).val() )

    if players_count < 4
      $('.players_count_4').hide()
    else
      $('.players_count_4').show()

    if players_count < 3
      $('.players_count_3').hide()
    else
      $('.players_count_3').show()



# Seems that bootstrap-toggle is not loaded by turbolinks
# So it is a good idea to interact with it to be sure it is loaded
$(document).on('turbolinks:load'
  ->
    if window.location.pathname == '/boards/new' || window.location.pathname.match( /boards\/\d+\/join/ )
      sauron_checkbox()
      match_players_count()
)