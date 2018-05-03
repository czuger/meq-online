require "application_system_test_case"

class CombatsTest < ApplicationSystemTestCase
  setup do
    @combat = combats(:one)
  end

  test "visiting the index" do
    visit combats_url
    assert_selector "h1", text: "Combats"
  end

  test "creating a Combat" do
    visit combats_url
    click_on "New Combat"

    fill_in "Board", with: @combat.board_id
    fill_in "Hero Cards Played", with: @combat.hero_cards_played
    fill_in "Hero", with: @combat.hero_id
    fill_in "Sauron Cards Played", with: @combat.sauron_cards_played
    fill_in "Temporary Strength", with: @combat.temporary_strength
    click_on "Create Combat"

    assert_text "Combat was successfully created"
    click_on "Back"
  end

  test "updating a Combat" do
    visit combats_url
    click_on "Edit", match: :first

    fill_in "Board", with: @combat.board_id
    fill_in "Hero Cards Played", with: @combat.hero_cards_played
    fill_in "Hero", with: @combat.hero_id
    fill_in "Sauron Cards Played", with: @combat.sauron_cards_played
    fill_in "Temporary Strength", with: @combat.temporary_strength
    click_on "Update Combat"

    assert_text "Combat was successfully updated"
    click_on "Back"
  end

  test "destroying a Combat" do
    visit combats_url
    page.accept_confirm do
      click_on "Destroy", match: :first
    end

    assert_text "Combat was successfully destroyed"
  end
end
