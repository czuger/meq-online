require 'ostruct'

module GameEngine
  class DataAtLocation

    attr_reader :tokens

    def initialize
      @characters = GameData::Characters.new
      @monsters = GameData::Mobs.new
      @tokens = {}
    end

    def gather(board)
      board.characters.each do |character, location|
        @tokens[location] ||= []
        @tokens[location] << OpenStruct.new( type: :character, code: character,
                                             name: @characters.name( character ), priority: 50,
                                             pic_path: "characters/small_#{character}.jpg".freeze )
      end

      board.current_plots.each do |plot|
        @tokens[plot.affected_location] ||= []
        @tokens[plot.affected_location] << OpenStruct.new( type: :plot, code: :plot, name: 'Plot', priority: 30,
                                             pic_path: 'plot_1.png'.freeze )

      end

      board.favors.each do |favor_location|
        @tokens[favor_location] ||= []
        @tokens[favor_location] << OpenStruct.new( type: :favor, code: :favor, name: 'Favor', priority: 100,
                                             pic_path: 'favor.png'.freeze )
      end

      board.heroes.each do |hero|
        @tokens[hero.location] ||= []
        @tokens[hero.location] << OpenStruct.new( type: :hero, code: hero.name_code, name: hero.name, priority: 10,
                                                   pic_path: "heroes_tokens/#{hero.name_code}.png".freeze )
      end

      board.monsters.each do |monster|
        @tokens[monster.location] ||= []
        @tokens[monster.location] << OpenStruct.new( type: :monster, code: monster.code,
          name: 'Monster', priority: 100,
          pic_path: "monsters/tokens/covers/#{monster.pool_key}.jpg".freeze,
          sauron_pic_path: "monsters/tokens/sauron_map/#{monster.code}.jpg".freeze,
          sauron_name: monster.name )
      end

      board.minions.each do |minion|
        @tokens[minion.location] ||= []
        @tokens[minion.location] << OpenStruct.new( type: :monster, code: minion.code,
                                                     name: board.monster_name(minion.code), priority: 40,
                                                     pic_path: "monsters/tokens/covers/#{minion.code}.png".freeze )
      end


      self
    end

  end
end