ENV['RAILS_ENV'] ||= 'test'

require 'simplecov'
SimpleCov.start 'rails'

require_relative '../config/environment'
require 'rails/test_help'
require 'mocha/minitest'

class ActiveSupport::TestCase
  # Setup all fixtures in test/fixtures/*.yml for all tests in alphabetical order.
  # fixtures :all

  # Add more helper methods to be used by all tests here...
  include FactoryBot::Syntax::Methods
end

def connection_for_tests(custom_user_uid = nil)
  auth_hash = {
    provider: 'discord',
    uid: 'foobar',
    info: {
      name: 'Foo Bar',
      email: 'foo_bar@gmail.com'
    },
    credentials: {
      token: 123456,
      expires_at: 'expire_time'
    }
  }

  auth_hash[:uid] = custom_user_uid if custom_user_uid

  OmniAuth.config.test_mode = true
  OmniAuth.config.mock_auth[:discord] = OmniAuth::AuthHash.new    auth_hash
  post '/auth/discord'
  follow_redirect!
end
