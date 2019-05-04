refresh= () ->

  App.refresh = App.cable.subscriptions.create "RefreshChannel",

    connected: ->
      # Called when the subscription is ready for use on the server

    disconnected: ->
      # Called when the subscription has been terminated by the server

    received: (data) ->
      # Called when there's incoming data on the websocket for this channel
  #    console.log(data)
      document.location.reload(false)

m= (path) ->
  window.location.pathname.match( path )

$(document).on('turbolinks:load'
  ->
    w_path = window.location.pathname
    if m( /maps\/\d+\/edit/ ) || m( /maps\/\d+/ ) || m( /boards/ )
      refresh()
)