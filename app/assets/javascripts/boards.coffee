# Place all the behaviors and hooks related to the matching controller here.
# All this logic will automatically be available in application.js.
# You can use CoffeeScript in this file: http://coffeescript.org/

# See : http://www.bootstraptoggle.com/
sauron_checkbox = () ->
  $('#sauron').bootstrapToggle({
    on: 'Yes',
    off: 'No'
  })

$(document).on('turbolinks:load'
  ->
    if window.location.pathname == '/boards/new'
      sauron_checkbox()
)