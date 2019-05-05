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
      plot = @@data[code.to_i]
      r = plot[:require]
      specific_requirement = r.nil? ? true : send( r, board )
      board.shadow_pool >= plot[:required_influence] && specific_requirement
    end

    def all_cards
      @@data.keys
    end

  end
end
