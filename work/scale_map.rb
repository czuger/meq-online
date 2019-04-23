require 'rmagick'
require 'fileutils'

map = Magick::Image.read('../app/assets/images/map.jpg')[0]

half_map = map.resize( 0.6 )
# fourth_map = map.resize( 0.25 )

# map.write('../app/assets/images/map_x4.jpg')
half_map.write('../app/assets/images/map_half.jpg')
# fourth_map.write('../app/assets/images/map_x1.jpg')