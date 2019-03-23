$(document).on('turbolinks:load'
  ->
    $('.toggle').bootstrapToggle({
      on: 'Yes',
      off: 'No'
    })
)