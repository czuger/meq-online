require "application_system_test_case"

class HerosTest < ApplicationSystemTestCase
  setup do
    @hero = heros(:one)
  end

  test "visiting the index" do
    visit heros_url
    assert_selector "h1", text: "Heros"
  end

  test "creating a Hero" do
    visit heros_url
    click_on "New Hero"

    fill_in "Agility", with: @hero.agility
    fill_in "Damage Pool", with: @hero.damage_pool
    fill_in "Fortitude", with: @hero.fortitude
    fill_in "Life Pool", with: @hero.life_pool
    fill_in "Location", with: @hero.location
    fill_in "Name Code", with: @hero.name_code
    fill_in "Rest Pool", with: @hero.rest_pool
    fill_in "Strength", with: @hero.strength
    fill_in "Wisdom", with: @hero.wisdom
    click_on "Create Hero"

    assert_text "Hero was successfully created"
    click_on "Back"
  end

  test "updating a Hero" do
    visit heros_url
    click_on "Edit", match: :first

    fill_in "Agility", with: @hero.agility
    fill_in "Damage Pool", with: @hero.damage_pool
    fill_in "Fortitude", with: @hero.fortitude
    fill_in "Life Pool", with: @hero.life_pool
    fill_in "Location", with: @hero.location
    fill_in "Name Code", with: @hero.name_code
    fill_in "Rest Pool", with: @hero.rest_pool
    fill_in "Strength", with: @hero.strength
    fill_in "Wisdom", with: @hero.wisdom
    click_on "Update Hero"

    assert_text "Hero was successfully updated"
    click_on "Back"
  end

  test "destroying a Hero" do
    visit heros_url
    page.accept_confirm do
      click_on "Destroy", match: :first
    end

    assert_text "Hero was successfully destroyed"
  end
end
