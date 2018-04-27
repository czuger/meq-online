json.extract! hero, :id, :name_code, :fortitude, :strength, :agility, :wisdom, :location, :life_pool, :rest_pool, :damage_pool, :created_at, :updated_at
json.url hero_url(hero, format: :json)
