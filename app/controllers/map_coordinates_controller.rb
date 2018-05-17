class MapCoordinatesController < ApplicationController

  def edit
    recorded_locations = nil
    File.open( 'work/data/places.txt', 'r' ) do |file|
      recorded_locations = file.readlines.map{ |l| l.split(':').first }
    end

    last_recorded_location =

    @locations = GameData::Locations.new
    recorded_locations.each do |loc|
      @locations.delete!( loc )
    end

    location = @locations.list_by_region.first

    @current_place_name = location.name
    @current_place_code = location.name_code

    @scroll_x = params['x']
    @scroll_y = params['y']
  end

  def update
    File.open( 'work/data/places.txt', 'a+' ) do |file|
      file.puts( "#{params[:current_place_code]}:#{params['true_x']},#{params['true_y']}")
    end
    redirect_to map_coordinates_edit_path( x: params['true_x'], y: params['true_y'] )
  end

end
