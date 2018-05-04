
module GameData
  class Base

    def select_tag_data
      @data.map{ |k, v| [ v[:name], k ] }
    end

  end
end
