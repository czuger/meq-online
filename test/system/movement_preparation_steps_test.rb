require "application_system_test_case"

class MovementPreparationStepsTest < ApplicationSystemTestCase
  setup do
    @movement_preparation_step = movement_preparation_steps(:one)
  end

  test "visiting the index" do
    visit movement_preparation_steps_url
    assert_selector "h1", text: "Movement Preparation Steps"
  end

  test "creating a Movement preparation step" do
    visit movement_preparation_steps_url
    click_on "New Movement Preparation Step"

    fill_in "Board", with: @movement_preparation_step.board_id
    fill_in "Card Used", with: @movement_preparation_step.card_used
    fill_in "Destination", with: @movement_preparation_step.destination
    fill_in "Hero", with: @movement_preparation_step.hero_id
    fill_in "Order", with: @movement_preparation_step.order
    fill_in "Origine", with: @movement_preparation_step.origine
    fill_in "Validation Required", with: @movement_preparation_step.validation_required
    click_on "Create Movement preparation step"

    assert_text "Movement preparation step was successfully created"
    click_on "Back"
  end

  test "updating a Movement preparation step" do
    visit movement_preparation_steps_url
    click_on "Edit", match: :first

    fill_in "Board", with: @movement_preparation_step.board_id
    fill_in "Card Used", with: @movement_preparation_step.card_used
    fill_in "Destination", with: @movement_preparation_step.destination
    fill_in "Hero", with: @movement_preparation_step.hero_id
    fill_in "Order", with: @movement_preparation_step.order
    fill_in "Origine", with: @movement_preparation_step.origine
    fill_in "Validation Required", with: @movement_preparation_step.validation_required
    click_on "Update Movement preparation step"

    assert_text "Movement preparation step was successfully updated"
    click_on "Back"
  end

  test "destroying a Movement preparation step" do
    visit movement_preparation_steps_url
    page.accept_confirm do
      click_on "Destroy", match: :first
    end

    assert_text "Movement preparation step was successfully destroyed"
  end
end
