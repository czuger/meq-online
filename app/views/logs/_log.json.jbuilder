json.extract! log, :id, :board_id, :action, :params, :created_at, :updated_at
json.url log_url(log, format: :json)
