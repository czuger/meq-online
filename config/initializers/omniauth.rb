OmniAuth.config.logger = Rails.logger

Rails.application.config.middleware.use OmniAuth::Builder do

  provider :developer if Rails.env.development?

  if File.exists?( 'config/omniauth.yml' )
    results = YAML.load( File.open( 'config/omniauth.yml' ).read )

    results.each do |result|
      provider *result
    end
  end
end

# OmniAuth.config.on_failure = Proc.new { |env|
#   OmniAuth::FailureEndpoint.new(env).redirect_to_failure
# }

if Rails.env.test?
  OmniAuth.config.full_host = 'http://localhost:3000'
end
