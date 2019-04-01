require 'ostruct'

module GameEngine
  class DataAtLocation

    attr_reader :tokens

    def initialize
      @characters = GameData::Characters.new
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
        @tokens[plot.affected_location] << OpenStruct.new( type: :plot, code: nil, name: 'Plot', priority: 10,
                                             pic_path: 'plot_1.png'.freeze )

      end

      board.favors.each do |favor_location|
        @tokens[favor_location] ||= []
        @tokens[favor_location] << OpenStruct.new( type: :favor, code: nil, name: 'Favor', priority: 100,
                                             pic_path: 'favor.png'.freeze )
      end

      self
    end

  end
end