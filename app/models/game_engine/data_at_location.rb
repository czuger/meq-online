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
        @tokens[location] << OpenStruct.new( pic_path: "characters/small_#{character}.jpg".freeze,
                                         text: @characters.name( character ) )
      end

      board.current_plots.each do |plot|
        @tokens[plot.affected_location] ||= []
        @tokens[plot.affected_location] << OpenStruct.new( pic_path: 'plot_1.png'.freeze, text: 'Plot' )
      end

      board.favors.each do |favor_location|
        @tokens[favor_location] ||= []
        @tokens[favor_location] << OpenStruct.new( pic_path: 'favor.png'.freeze, text: 'Favor' )
      end

      self
    end

  end
end