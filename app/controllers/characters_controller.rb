class CharactersController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_board, only: [:show, :edit, :update]

  def show
  end

  def edit
  end

  def update
  end
end
