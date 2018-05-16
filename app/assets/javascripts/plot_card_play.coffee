# Place all the behaviors and hooks related to the matching controller here.
# All this logic will automatically be available in application.js.
# You can use CoffeeScript in this file: http://coffeescript.org/

selected_cards = {}

$(document).on('turbolinks:load'
  ->
    if window.location.pathname.match( /plot_card_play\/\d+\/edit/ )
      $('.draggable-card').draggable();
      $('.plot-card-on-board').droppable drop: (event, ui) ->

        $(this).html("<img class='big-card' src=#{ui.draggable[0].src}/>")
        ui.draggable[0].remove()

        card = $(ui.draggable[0])
        selected_cards[$(this).attr('id')] = card.attr('card_id')
        $('#selected_cards').val( JSON.stringify(selected_cards) )
        return

)