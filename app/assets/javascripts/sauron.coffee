# Place all the behaviors and hooks related to the matching controller here.
# All this logic will automatically be available in application.js.
# You can use CoffeeScript in this file: http://coffeescript.org/

influence= () ->
  console.log('toto')
  $('.sauron_action' ).change () ->
    console.log($(this))

    if $( "input:checked" ).length >= 4
      console.log( $( "input:checked" ).length )
      $('.sauron_action' ).prop( "checked", false );
      $(this).prop( "checked", true );

    console.log( jQuery.makeArray( $("input:checked") ).map (e) -> e.id )

#activate_tooltips= () ->
#  $('[data-toggle="tooltip"]').tooltip()

$(document).on('turbolinks:load'
  ->
    console.log('toto1')
    if window.location.pathname.match( /sauron_actions\/\d+\/edit/ )
      influence()
)