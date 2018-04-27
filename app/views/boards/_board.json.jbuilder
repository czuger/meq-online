json.extract! board, :id, :heroes, :created_at, :updated_at
json.url board_url(board, format: :json)
