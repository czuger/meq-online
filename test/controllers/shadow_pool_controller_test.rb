require 'test_helper'

class ShadowPoolControllerTest < ActionDispatch::IntegrationTest
  test "should get edit" do
    get shadow_pool_edit_url
    assert_response :success
  end

  test "should get update" do
    get shadow_pool_update_url
    assert_response :success
  end

end
