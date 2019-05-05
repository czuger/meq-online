# This module is included in GameData::Plot.

module GameEngine
  module PlotRequirements

    private

    #
    # Cards power
    #
    def minion_in_erebor( board )
      board.minions.where( location: 'erebor'.freeze ).exists?
    end

    def monster_in_osgiliath( board )
      board.monsters.where( location: 'osgiliath'.freeze ).exists?
    end

    def influence_near_edoras( board )
      %w( edoras plains_of_rhoan the_nidale dunharrow helms_deep ).map{ |e| nil_to_zero(board.influence[e]) }.inject(&:+) >= 6 ||
          board.current_plots.where(plot_card: 6).exists?
    end

    def influence_in_isengard( board )
      nil_to_zero(board.influence['isengard'.freeze]) >= 3
    end

    def influence_near_mount_gundabad( board )
      %w( mount_gundabad the_ruins_of_angmar the_northern_waste the_high_pass ).map{ |e| nil_to_zero(board.influence[e]) }.inject(&:+) >= 8
    end

    def require_mob_near_wodden_land( board )
      board.mobs.where( location: %w( the_forest_trail the_forest_road lake_esgaroth ) ).exists?
    end

    def nil_to_zero( value )
      value || 0
    end

  end
end
