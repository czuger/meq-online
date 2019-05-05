require 'yaml'
require 'ostruct'

module GameData
  class Locations < Base

    REGIONAL_HAVEN_COLORS_CONVERSIONS = { yellow: :light_green, purple: :orange, red: :light_blue, brown: :light_blue }

    def get( name_code )
      OpenStruct.new fast_get(name_code )
    end

    def all_codes
      @data.keys
    end

    def perilous?( name_code )
      get(name_code).perilous
    end

    def can_not_rest?( name_code )
      fast_get(name_code)[:shadow_fortress]
    end

    def can_heal?( name_code )
      fast_get(name_code)[:haven]
    end

    def exist?( name_code )
      @data.has_key?( name_code )
    end

    def delete!( location_name )
      @data.delete( location_name )
    end

    def list_by_region
      @data.map{ |k, v| OpenStruct.new( name_code: k, name: v[:name], region: v[:region] ) }.sort_by { |e| [e.region, e.name ] }
    end

    def position_list
      @data.map{ |k, v| OpenStruct.new( name_code: k, x: v[:pos_x], y: v[:pos_y], haven: v[:haven] ) }
    end

    def alpha_select_tag_data
      @data.map{ |k, v| [ v[:name], k ] }.sort
    end

    def get_haven_for_color(color)
      color = REGIONAL_HAVEN_COLORS_CONVERSIONS[color] || color
      @data.each do |k, v|
        return k if v[:color_code] == color && v[:haven] == true
      end
      raise "Could not find haven for #{color}"
    end

    private

    def fast_get( name_code )
      raise "Location '#{name_code.inspect}' not found" unless @data.has_key?(name_code.to_s)
      @data[name_code.to_s]
    end

  end
end
