module GameData
  class HeroQuests < Base

    FILENAME = 'heroes_quests'

    # quest_set = regular, extension_1, etc ...
    def initialize( quests_set )
      super()
      @quests_set = quests_set.to_sym
    end

    def get_starting_quest( hero_name_code )
      @data[@quests_set][hero_name_code.to_sym][:starting_quests].sample
    end

    def hero_setup( board, hero_name_code )
      setup_array = get_starting_quest_setup_array( hero_name_code )
      if setup_array
        setup_array.each do |setup_element|
          if setup_element[:action] == :place
            board.characters[setup_element[:character]] = setup_element[:location]
          else
            raise "Unknown action #{setup_element[:action]}"
          end
        end
        board.save!
      end
    end

    private

    def get_starting_quest_setup_array( hero_name_code )
      sq_number = get_starting_quest( hero_name_code )
      @data[@quests_set][hero_name_code.to_sym][:setup][sq_number]
    end

  end
end
