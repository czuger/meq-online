module GameData
  class Plots < Base

    include GameEngine::PlotRequirements

    @@data = nil

    def initialize
      super
      @@data ||= @data
      remove_instance_variable(:@data)
    end

    def get( code )
      code = code.to_i

      @@data[code][:influence] = OpenStruct.new( @@data[code][:influence] ) unless @@data[code][:influence].is_a? OpenStruct
      @@data[code][:shadow_cards] = OpenStruct.new( @@data[code][:shadow_cards] ) unless @@data[code][:shadow_cards].is_a? OpenStruct
      @@data[code][:plot_cards] = OpenStruct.new( @@data[code][:plot_cards] ) unless @@data[code][:plot_cards].is_a? OpenStruct
      @@data[code][:story] = OpenStruct.new( @@data[code][:story] ) unless @@data[code][:story].is_a? OpenStruct

      OpenStruct.new( @@data[code] )
    end

    def requirement( board, code )
      r = @@data[code.to_i][:require]
      return true if r.nil?
      send( r, board )
    end

    def all_cards
      @@data.keys
    end

  end
end
