module CharactersHelper

  def character_code( char )
    "characters[#{char.name_code}]"
  end

  def local_options_for_select( char )
    char_location = @board.characters[char.name_code]
    options_for_select( @locations.alpha_select_tag_data, char_location )
  end

end
