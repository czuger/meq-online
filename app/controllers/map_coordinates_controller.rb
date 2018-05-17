class MapCoordinatesController < ApplicationController

  def edit
    @locations = GameData::Locations.new.list_by_region

    location = @locations.first

    @current_place_name = location.name
    @current_place_code = location.name_code
  end

  def update
    File.open( 'work/data/places.txt', 'a+' ) do |file|
      file.puts( "#{params[:current_place_code]}:#{params['true_x']},#{params['true_y']}")
    end
    redirect_to map_coordinates_edit_path
  end

end
