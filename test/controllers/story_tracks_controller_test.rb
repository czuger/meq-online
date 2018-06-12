require 'test_helper'

class StoryTracksControllerTest < ActionDispatch::IntegrationTest
  test "should get edit" do
    get story_tracks_edit_url
    assert_response :success
  end

  test "should get update" do
    get story_tracks_update_url
    assert_response :success
  end

end
