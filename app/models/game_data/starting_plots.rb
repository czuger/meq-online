module GameData
  class StartingPlots < Base

    FILENAME = 'starting_plots'

    def get( code )
      @data[code][:influence] = OpenStruct.new( @data[code][:influence] ) unless @data[code][:influence].is_a? OpenStruct
      @data[code][:shadow_cards] = OpenStruct.new( @data[code][:shadow_cards] ) unless @data[code][:shadow_cards].is_a? OpenStruct
      @data[code][:plot_cards] = OpenStruct.new( @data[code][:plot_cards] ) unless @data[code][:plot_cards].is_a? OpenStruct
      OpenStruct.new( @data[code] )
    end

    def sauron_setup( board )

    end

  end
end
