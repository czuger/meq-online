require 'rmagick'
require 'fileutils'

require_relative 'libs/round.rb'

directory = 'hero_cards/heroes'

decal_x = {
    'beravor.jpg' => 30, 'eometh.jpg' => 20, 'thalin.jpg' => 50
}

decal_y = {
    'beravor.jpg' => 13
}

Dir.entries( directory ).each do |f|
  p = directory + '/' + f
  next if File.directory?(p)
  # p p
  img = Magick::Image.read(p)[0]

  top = 100
  top += decal_x[f] if decal_x[f]

  left = 125
  left += decal_y[f] if decal_y[f]

  new_image_name = directory + '/small/' + File.basename( f, '.jpg' ) + '.png'

  new_img = img.crop(left, top, 92, 92)

  new_img = round( new_img, 20 )

  new_img.write( new_image_name )
end



