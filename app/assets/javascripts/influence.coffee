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

$(document).on('turbolinks:load'
  ->
    if window.location.pathname.match( /influences\/\d+\/edit/ )
      filter()
)