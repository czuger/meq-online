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
    # Refresh for boards and other screen leads to self refresh issues.
    if m( /maps\/\d+\/edit/ ) || m( /maps\/\d+/ )
      refresh()
)