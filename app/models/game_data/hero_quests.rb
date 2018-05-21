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

  end
end
