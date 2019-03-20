class CharactersController < ApplicationController

  before_action :require_logged_in
  before_action :set_actor_ensure_board, only: [:show, :edit, :update]
  before_action :set_characters, only: [:update, :edit]

  def edit
  end

  def update
    ActionController::Parameters.permit_all_parameters = true
    characters = params[:characters].to_h

    @board.transaction do

      characters.each do |char, location|
        char = char

        if location.empty?
          if @board.characters[char]
            @board.characters[char] = nil
            @board.log( @actor, 'character.remove', name: @characters.name( char ) )
          end
        else
          next unless @locations.exist?(location)
          @board.characters[char] = location.to_sym
          @board.log( @actor, 'character.place', name: @characters.name( char ),
                       location: @locations.get( location ).name )
        end
      end

      @board.save!
    end

    redirect_to edit_character_path(@actor)
  end

  private

  def set_characters
    @locations= GameData::Locations.new
    @characters = GameData::Characters.new
  end

end
