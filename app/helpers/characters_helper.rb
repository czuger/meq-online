module CharactersHelper

  def character_code( char )
    "characters[#{char.name_code}]"
  end
end
