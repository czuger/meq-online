require 'ostruct'

module GameEngine
  class DataAtLocation

    attr_reader :tokens

    def initialize
      @characters = GameData::Characters.new
      @monsters = GameData::Monsters.new
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

      board.monsters_on_board.each do |location, monsters_list|
        monsters_list.each do |monster_hash|
          @tokens[location] ||= []
          @tokens[location] << OpenStruct.new( type: :monster, code: monster_hash['monster'],
            name: 'Monster', priority: 100,
            pic_path: "monsters/tokens/covers/#{monster_hash['monster_pool_key']}.jpg".freeze,
            private_pic_path: nil, private_name: board.monster_name(monster_hash['monster'] ) )
        end
      end

      self
    end

  end
end